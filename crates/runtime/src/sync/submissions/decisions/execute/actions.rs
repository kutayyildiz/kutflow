use super::super::Planned;
use crate::{
    request::ResolvedAction,
    runtime::{Runtime, RuntimeError},
    transition::TransitionExt,
};
use kutflow_core::{
    Transition,
    components::{Commit, Discarded, TransitionDecision, TransitionReason, Transitioned, Wish},
};

pub(super) fn apply(
    runtime: &mut Runtime,
    target: hecs::Entity,
    plan: Vec<Planned>,
) -> Result<(), RuntimeError> {
    runtime.stage_for.remove(&target);
    runtime.actor_for.remove(&target);
    for action in plan {
        match action {
            Planned::Transition(decision) => {
                apply_transition(runtime, &decision)?;
                runtime
                    .history
                    .entry(target)
                    .or_default()
                    .push(decision.reason);
            }
            Planned::Mark(decision) => {
                if runtime.world.get::<&Discarded>(target).is_err() {
                    runtime
                        .world
                        .insert(
                            target,
                            (
                                TransitionDecision(decision.transition),
                                TransitionReason(decision.reason.clone()),
                            ),
                        )
                        .map_err(|_| RuntimeError::InvalidResponse)?;
                }
                runtime
                    .history
                    .entry(target)
                    .or_default()
                    .push(decision.reason);
            }
            Planned::Next {
                stage,
                actor,
                reason,
            } => {
                runtime.stage_for.insert(target, stage);
                if let Some(reason) = reason {
                    runtime.history.entry(target).or_default().push(reason);
                }
                if let Some(actor) = actor {
                    runtime.actor_for.insert(target, actor);
                    if let Ok(name) = runtime.world.get::<&kutflow_core::components::Name>(actor) {
                        runtime
                            .history
                            .entry(target)
                            .or_default()
                            .push(format!("Assigned to actor {}.", name.0));
                    }
                }
            }
            Planned::Generate {
                items,
                wishes,
                next,
            } => generate(runtime, target, items, wishes, next.as_deref())?,
            Planned::Delegate { .. } => return Err(RuntimeError::InvalidResponse),
        }
    }
    Ok(())
}

fn generate(
    runtime: &mut Runtime,
    target: hecs::Entity,
    items: Vec<String>,
    wishes: bool,
    next: Option<&str>,
) -> Result<(), RuntimeError> {
    if runtime.world.get::<&Discarded>(target).is_err() {
        runtime
            .world
            .insert_one(target, Transitioned)
            .map_err(|_| RuntimeError::InvalidResponse)?;
    }
    let history = runtime.history.get(&target).cloned().unwrap_or_default();
    for item in items {
        let entity = if wishes {
            runtime.world.spawn((Wish(item),))
        } else {
            runtime.world.spawn((Commit(item),))
        };
        if let Some(stage) = next {
            runtime.stage_for.insert(entity, stage.to_owned());
        }
        let mut output_history = history.clone();
        if let Ok(item) = runtime.world.get::<&Wish>(entity) {
            output_history.push(item.0.clone());
        } else if let Ok(item) = runtime.world.get::<&Commit>(entity) {
            output_history.push(item.0.clone());
        }
        runtime.history.insert(entity, output_history);
    }
    Ok(())
}

fn apply_transition(runtime: &mut Runtime, action: &ResolvedAction) -> Result<(), RuntimeError> {
    if matches!(
        action.transition,
        Transition::AddWish | Transition::AddCommit
    ) {
        return Err(RuntimeError::InvalidResponse);
    }
    runtime
        .world
        .insert(
            action.target,
            (
                TransitionDecision(action.transition),
                TransitionReason(action.reason.clone()),
            ),
        )
        .map_err(|_| RuntimeError::InvalidResponse)?;
    if action.transition == Transition::Review {
        runtime
            .world
            .insert_one(action.target, Transitioned)
            .map_err(|_| RuntimeError::InvalidResponse)?;
    } else {
        action
            .transition
            .apply(&mut runtime.world, &[action.target])?;
    }
    Ok(())
}
