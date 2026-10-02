use kutflow_core::components::{Commit, Data, Discarded, Transitioned, Wish};

use crate::{
    protocol::Response,
    request::{Pending, PendingAction},
    runtime::{Runtime, RuntimeError},
};

mod decisions;

pub(super) fn apply(runtime: &mut Runtime) -> Result<(), RuntimeError> {
    let mut pending = std::mem::take(&mut runtime.pending);
    while !pending.is_empty() {
        let item = pending.remove(0);
        let Some(response) = item.response.clone() else {
            runtime.pending.push(item);
            continue;
        };
        if let Err(error) = apply_one(runtime, &item, response) {
            runtime.pending.push(item);
            runtime.pending.extend(pending);
            return Err(error);
        }
        let discarded = item
            .targets()
            .iter()
            .copied()
            .filter(|target| runtime.world.get::<&Discarded>(*target).is_ok())
            .collect::<Vec<_>>();
        pending.retain(|other| {
            matches!(other.action, PendingAction::Generation { .. })
                || !other
                    .targets()
                    .iter()
                    .any(|target| discarded.contains(target))
        });
        runtime.pending.retain(|other| {
            matches!(other.action, PendingAction::Generation { .. })
                || !other
                    .targets()
                    .iter()
                    .any(|target| discarded.contains(target))
        });
    }
    Ok(())
}

fn apply_one(
    runtime: &mut Runtime,
    item: &Pending,
    response: Response,
) -> Result<(), RuntimeError> {
    if !item.request.accepts(&response) {
        return Err(RuntimeError::InvalidResponse);
    }
    match (&item.action, response) {
        (PendingAction::Decisions, Response::Decisions(decisions)) => {
            decisions::apply(runtime, &decisions)
        }
        (
            PendingAction::Generation {
                transition,
                targets,
            },
            Response::Wishes(values),
        ) if *transition == kutflow_core::Transition::AddWish => {
            validate_generation_targets(runtime, targets)?;
            for value in values {
                runtime.world.spawn((Wish(value),));
            }
            complete_generation(runtime, targets)
        }
        (
            PendingAction::Generation {
                transition,
                targets,
            },
            Response::Commits(values),
        ) if *transition == kutflow_core::Transition::AddCommit => {
            validate_generation_targets(runtime, targets)?;
            for value in values {
                runtime.world.spawn((Commit(value),));
            }
            complete_generation(runtime, targets)
        }
        _ => Err(RuntimeError::InvalidResponse),
    }
}

fn validate_generation_targets(
    runtime: &Runtime,
    targets: &[hecs::Entity],
) -> Result<(), RuntimeError> {
    for target in targets {
        runtime
            .world
            .entity(*target)
            .map_err(|_| RuntimeError::InvalidResponse)?;
        if runtime.world.get::<&Data>(*target).is_ok() {
            return Err(RuntimeError::InvalidResponse);
        }
    }
    Ok(())
}

fn complete_generation(
    runtime: &mut Runtime,
    targets: &[hecs::Entity],
) -> Result<(), RuntimeError> {
    for target in targets {
        if runtime.world.get::<&Discarded>(*target).is_err() {
            runtime
                .world
                .insert_one(*target, Transitioned)
                .map_err(|_| RuntimeError::InvalidResponse)?;
        }
    }
    Ok(())
}
