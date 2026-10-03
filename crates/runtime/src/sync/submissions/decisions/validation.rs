use hecs::Entity;
use kutflow_core::{
    Transition,
    components::{BelongsToBranch, Branch, Commit, Data, Discarded, Wish},
};

use crate::{
    request::ResolvedAction,
    runtime::{Runtime, RuntimeError},
};

pub(super) fn validate(
    runtime: &Runtime,
    decisions: &[ResolvedAction],
) -> Result<(), RuntimeError> {
    for decision in decisions {
        let target = decision.target;
        match decision.transition {
            Transition::Review => {
                active(runtime, target)?;
                if !has_item(runtime, target) {
                    return Err(RuntimeError::InvalidResponse);
                }
            }
            Transition::AddWish | Transition::AddCommit => {
                active(runtime, target)?;
                if !(runtime.world.get::<&Wish>(target).is_ok()
                    || runtime.world.get::<&Commit>(target).is_ok())
                {
                    return Err(RuntimeError::InvalidResponse);
                }
            }
            Transition::DiscardWish => item::<Wish>(runtime, target)?,
            Transition::DiscardCommit => item::<Commit>(runtime, target)?,
            Transition::CreateBranch => return Err(RuntimeError::InvalidResponse),
            Transition::DiscardBranch => item::<Branch>(runtime, target)?,
        }
    }
    validate_branch_effects(runtime, decisions)
}

fn validate_branch_effects(
    runtime: &Runtime,
    decisions: &[ResolvedAction],
) -> Result<(), RuntimeError> {
    let discarded = decisions
        .iter()
        .filter(|decision| decision.transition == Transition::DiscardBranch)
        .map(|decision| decision.target)
        .collect::<Vec<_>>();
    let affected = runtime
        .world
        .query::<(Entity, &BelongsToBranch)>()
        .iter()
        .filter_map(|(entity, belongs)| discarded.contains(&belongs.0).then_some(entity))
        .collect::<Vec<_>>();
    if !discarded.is_empty()
        && decisions.iter().any(|decision| {
            decision.transition != Transition::DiscardBranch && affected.contains(&decision.target)
        })
    {
        return Err(RuntimeError::InvalidResponse);
    }
    Ok(())
}

fn item<T: hecs::Component>(runtime: &Runtime, target: Entity) -> Result<(), RuntimeError> {
    active(runtime, target)?;
    runtime
        .world
        .get::<&T>(target)
        .map(|_| ())
        .map_err(|_| RuntimeError::InvalidResponse)
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
