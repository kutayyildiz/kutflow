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
            .target()
            .filter(|target| runtime.world.get::<&Discarded>(*target).is_ok());
        pending.retain(|other| {
            matches!(other.action, PendingAction::Generation { .. }) || other.target() != discarded
        });
        runtime.pending.retain(|other| {
            matches!(other.action, PendingAction::Generation { .. }) || other.target() != discarded
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
        (PendingAction::Answers(reasons), Response::Answers(answers)) => {
            let decisions = crate::request::resolve(reasons, &answers)
                .map_err(|_| RuntimeError::InvalidResponse)?;
            decisions::apply(runtime, &decisions)
        }
        (PendingAction::Generation { transition, target }, Response::Wishes(values))
            if *transition == kutflow_core::Transition::AddWish =>
        {
            validate_generation_target(runtime, *target)?;
            for value in values {
                runtime.world.spawn((Wish(value),));
            }
            complete_generation(runtime, *target)
        }
        (PendingAction::Generation { transition, target }, Response::Commits(values))
            if *transition == kutflow_core::Transition::AddCommit =>
        {
            validate_generation_target(runtime, *target)?;
            for value in values {
                runtime.world.spawn((Commit(value),));
            }
            complete_generation(runtime, *target)
        }
        _ => Err(RuntimeError::InvalidResponse),
    }
}

fn validate_generation_target(runtime: &Runtime, target: hecs::Entity) -> Result<(), RuntimeError> {
    runtime
        .world
        .entity(target)
        .map_err(|_| RuntimeError::InvalidResponse)?;
    if runtime.world.get::<&Data>(target).is_ok() {
        return Err(RuntimeError::InvalidResponse);
    }
    Ok(())
}

fn complete_generation(runtime: &mut Runtime, target: hecs::Entity) -> Result<(), RuntimeError> {
    if runtime.world.get::<&Discarded>(target).is_err() {
        runtime
            .world
            .insert_one(target, Transitioned)
            .map_err(|_| RuntimeError::InvalidResponse)?;
    }
    Ok(())
}
