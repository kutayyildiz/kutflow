use hecs::{Entity, World};
use kutflow_core::components::{
    BelongsToBranch, Branch, Commit, TransitionDecision, TransitionReason, Transitioned,
};

use super::{TransitionError, helpers::one};

pub(super) fn create(world: &mut World, commits: &[Entity]) -> Result<(), TransitionError> {
    if commits.len() < 2 {
        return Err(TransitionError::ExpectedAtLeastTwoTargets);
    }
    for entity in commits {
        world
            .get::<&Commit>(*entity)
            .map_err(|_| TransitionError::MissingComponent("Commit"))?;
        if world.get::<&BelongsToBranch>(*entity).is_ok() {
            return Err(TransitionError::AlreadyBelongsToBranch);
        }
    }
    let branch = world.spawn((Branch,));
    for entity in commits {
        world
            .insert_one(*entity, BelongsToBranch(branch))
            .map_err(|_| TransitionError::InvalidEntity)?;
    }
    Ok(())
}

pub(super) fn discard(world: &mut World, targets: &[Entity]) -> Result<(), TransitionError> {
    let branch = one(targets)?;
    world
        .get::<&Branch>(branch)
        .map_err(|_| TransitionError::MissingComponent("Branch"))?;
    let commits = world
        .query::<(Entity, &BelongsToBranch)>()
        .iter()
        .filter_map(|(entity, belongs)| (belongs.0 == branch).then_some(entity))
        .collect::<Vec<_>>();
    for entity in commits {
        world
            .remove_one::<BelongsToBranch>(entity)
            .map_err(|_| TransitionError::InvalidEntity)?;
        let _ = world.remove_one::<TransitionDecision>(entity);
        let _ = world.remove_one::<TransitionReason>(entity);
        let _ = world.remove_one::<Transitioned>(entity);
    }
    Ok(())
}
