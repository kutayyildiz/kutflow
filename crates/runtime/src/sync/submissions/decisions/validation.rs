use hecs::Entity;
use kutflow_core::{
    Transition,
    components::{BelongsToBranch, Branch, Commit, Data, Discarded, Wish},
};

use crate::{
    protocol::Decision,
    runtime::{Runtime, RuntimeError},
};

pub(super) fn validate(runtime: &Runtime, decisions: &[Decision]) -> Result<(), RuntimeError> {
    for decision in decisions {
        validate_action(runtime, decision)?;
    }
    validate_branch_effects(runtime, decisions)
}

fn validate_action(runtime: &Runtime, decision: &Decision) -> Result<(), RuntimeError> {
    match decision.transition {
        Transition::Review => {
            for target in &decision.targets {
                active(runtime, *target)?;
                if !has_item(runtime, *target) {
                    return Err(RuntimeError::InvalidResponse);
                }
            }
        }
        Transition::AddWish | Transition::AddCommit => {
            for target in &decision.targets {
                active(runtime, *target)?;
                if !(runtime.world.get::<&Wish>(*target).is_ok()
                    || runtime.world.get::<&Commit>(*target).is_ok())
                {
                    return Err(RuntimeError::InvalidResponse);
                }
            }
        }
        Transition::DiscardWish => items::<Wish>(runtime, &decision.targets)?,
        Transition::DiscardCommit => items::<Commit>(runtime, &decision.targets)?,
        Transition::CreateBranch => {
            if decision.targets.len() < 2 {
                return Err(RuntimeError::InvalidResponse);
            }
            items::<Commit>(runtime, &decision.targets)?;
            for target in &decision.targets {
                if runtime.world.get::<&BelongsToBranch>(*target).is_ok() {
                    return Err(RuntimeError::InvalidResponse);
                }
            }
        }
        Transition::DiscardBranch => {
            if decision.targets.len() != 1 {
                return Err(RuntimeError::InvalidResponse);
            }
            items::<Branch>(runtime, &decision.targets)?;
        }
    }
    Ok(())
}

fn validate_branch_effects(runtime: &Runtime, decisions: &[Decision]) -> Result<(), RuntimeError> {
    let discarded = decisions
        .iter()
        .filter(|d| d.transition == Transition::DiscardBranch)
        .flat_map(|d| d.targets.iter().copied())
        .collect::<Vec<_>>();
    let affected = runtime
        .world
        .query::<(Entity, &BelongsToBranch)>()
        .iter()
        .filter_map(|(entity, belongs)| discarded.contains(&belongs.0).then_some(entity))
        .collect::<Vec<_>>();
    if !discarded.is_empty()
        && decisions.iter().any(|decision| {
            decision.transition != Transition::DiscardBranch
                && decision
                    .targets
                    .iter()
                    .any(|target| affected.contains(target))
        })
    {
        return Err(RuntimeError::InvalidResponse);
    }
    Ok(())
}

fn items<T: hecs::Component>(runtime: &Runtime, targets: &[Entity]) -> Result<(), RuntimeError> {
    for target in targets {
        active(runtime, *target)?;
        if runtime.world.get::<&T>(*target).is_err() {
            return Err(RuntimeError::InvalidResponse);
        }
    }
    Ok(())
}

fn active(runtime: &Runtime, target: Entity) -> Result<(), RuntimeError> {
    runtime
        .world
        .entity(target)
        .map_err(|_| RuntimeError::InvalidResponse)?;
    if runtime.world.get::<&Data>(target).is_ok() || runtime.world.get::<&Discarded>(target).is_ok()
    {
        return Err(RuntimeError::InvalidResponse);
    }
    Ok(())
}

fn has_item(runtime: &Runtime, target: Entity) -> bool {
    runtime.world.get::<&Wish>(target).is_ok()
        || runtime.world.get::<&Commit>(target).is_ok()
        || runtime.world.get::<&Branch>(target).is_ok()
}
