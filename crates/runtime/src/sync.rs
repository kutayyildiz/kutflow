use hecs::Entity;
use kutflow_core::{
    components::{Data, TransitionDecision, Transitioned},
    views::View,
};

use crate::{
    request::PendingAction,
    runtime::{Runtime, RuntimeError},
    transition::TransitionExt,
};

pub(super) fn sync(runtime: &mut Runtime) -> Result<(), RuntimeError> {
    apply_submissions(runtime)?;
    prepare_transitions(runtime)?;
    super::rules::prepare_rules(runtime)
}

fn apply_submissions(runtime: &mut Runtime) -> Result<(), RuntimeError> {
    let mut pending = std::mem::take(&mut runtime.pending).into_iter();
    while let Some(item) = pending.next() {
        if item.response.is_none() {
            runtime.pending.push(item);
            continue;
        }
        match item.apply(&mut runtime.world) {
            Ok(Some((transition, targets))) => {
                if let Err(error) = schedule_transition(runtime, transition, targets) {
                    runtime.pending.push(item);
                    runtime.pending.extend(pending);
                    return Err(error);
                }
            }
            Ok(None) => {}
            Err(error) => {
                runtime.pending.push(item);
                runtime.pending.extend(pending);
                return Err(error.into());
            }
        }
    }
    Ok(())
}

fn prepare_transitions(runtime: &mut Runtime) -> Result<(), RuntimeError> {
    let entities = runtime
        .world
        .query::<(Entity, &TransitionDecision)>()
        .iter()
        .filter_map(|(entity, decision)| {
            (!runtime.world.get::<&Data>(entity).is_ok()
                && runtime.world.get::<&Transitioned>(entity).is_err())
            .then_some((entity, decision.0))
        })
        .collect::<Vec<_>>();
    let branch_targets = entities
        .iter()
        .filter_map(|(entity, transition)| {
            (*transition == kutflow_core::Transition::CreateBranch
                && is_current_target(runtime, *entity, *transition)
                && !super::rules::is_pending(runtime, *entity))
            .then_some(*entity)
        })
        .collect::<Vec<_>>();
    if !branch_targets.is_empty() {
        schedule_transition(
            runtime,
            kutflow_core::Transition::CreateBranch,
            branch_targets,
        )?;
    }
    for (entity, transition) in entities {
        if transition == kutflow_core::Transition::CreateBranch
            || !is_current_target(runtime, entity, transition)
            || super::rules::is_pending(runtime, entity)
        {
            continue;
        }
        schedule_transition(runtime, transition, vec![entity])?;
    }
    Ok(())
}

fn is_current_target(
    runtime: &Runtime,
    entity: Entity,
    transition: kutflow_core::Transition,
) -> bool {
    runtime
        .world
        .get::<&TransitionDecision>(entity)
        .is_ok_and(|decision| decision.0 == transition)
        && runtime.world.get::<&Transitioned>(entity).is_err()
        && runtime.world.get::<&Data>(entity).is_err()
}

fn schedule_transition(
    runtime: &mut Runtime,
    transition: kutflow_core::Transition,
    targets: Vec<Entity>,
) -> Result<(), RuntimeError> {
    let context = target_context(runtime, transition, &targets)?;
    if let Some((prompt, output)) = transition.request() {
        runtime.request(
            context,
            prompt,
            output,
            PendingAction::Transition {
                transition,
                targets,
            },
        );
    } else {
        transition.apply(&mut runtime.world, &targets, None)?;
    }
    Ok(())
}

fn target_context(
    runtime: &Runtime,
    transition: kutflow_core::Transition,
    targets: &[Entity],
) -> Result<String, RuntimeError> {
    match transition {
        kutflow_core::Transition::ModifyWish
        | kutflow_core::Transition::DissectWish
        | kutflow_core::Transition::DiscardWish
        | kutflow_core::Transition::AddCommit => {
            Ok(View::Wishes.read(runtime.world(), targets)?.join("\n"))
        }
        kutflow_core::Transition::ModifyCommit
        | kutflow_core::Transition::DissectCommit
        | kutflow_core::Transition::DiscardCommit => {
            Ok(View::Commits.read(runtime.world(), targets)?.join("\n"))
        }
        _ => Ok(String::new()),
    }
}
