use hecs::{Entity, World};
use kutflow_core::{
    Transition,
    components::{Commit, Discarded, Wish},
};

mod branch;
mod error;
mod helpers;
pub use error::TransitionError;

pub trait TransitionExt {
    fn apply(&self, world: &mut World, targets: &[Entity]) -> Result<(), TransitionError>;
}

impl TransitionExt for Transition {
    fn apply(&self, world: &mut World, targets: &[Entity]) -> Result<(), TransitionError> {
        validate_targets(world, *self, targets)?;
        match self {
            Self::Review | Self::AddWish | Self::AddCommit => {
                return Err(TransitionError::RequiresRequest);
            }
            Self::DiscardWish => discard::<Wish>(world, targets, "Wish")?,
            Self::DiscardCommit => discard::<Commit>(world, targets, "Commit")?,
            Self::CreateBranch => branch::create(world, targets)?,
            Self::DiscardBranch => {
                branch::discard(world, targets)?;
                let entity = helpers::one(targets)?;
                world
                    .despawn(entity)
                    .map_err(|_| TransitionError::InvalidEntity)?;
            }
        }
        Ok(())
    }
}

fn validate_targets(
    world: &World,
    transition: Transition,
    targets: &[Entity],
) -> Result<(), TransitionError> {
    for entity in targets {
        world
            .entity(*entity)
            .map_err(|_| TransitionError::InvalidEntity)?;
        if world
            .get::<&kutflow_core::components::Discarded>(*entity)
            .is_ok()
            || world
                .get::<&kutflow_core::components::Data>(*entity)
                .is_ok()
        {
            return Err(TransitionError::InvalidEntity);
        }
        let valid = match transition {
            Transition::AddWish | Transition::AddCommit => {
                world.get::<&Wish>(*entity).is_ok() || world.get::<&Commit>(*entity).is_ok()
            }
            Transition::DiscardWish => world.get::<&Wish>(*entity).is_ok(),
            Transition::DiscardCommit | Transition::CreateBranch => {
                world.get::<&Commit>(*entity).is_ok()
            }
            Transition::DiscardBranch => world
                .get::<&kutflow_core::components::Branch>(*entity)
                .is_ok(),
            Transition::Review => {
                world.get::<&Wish>(*entity).is_ok()
                    || world.get::<&Commit>(*entity).is_ok()
                    || world
                        .get::<&kutflow_core::components::Branch>(*entity)
                        .is_ok()
            }
        };
        if !valid {
            return Err(TransitionError::MissingComponent("workflow item"));
        }
    }
    Ok(())
}

fn discard<T: hecs::Component>(
    world: &mut World,
    targets: &[Entity],
    component: &'static str,
) -> Result<(), TransitionError> {
    for entity in targets {
        if world.get::<&T>(*entity).is_err() {
            return Err(TransitionError::MissingComponent(component));
        }
    }
    for entity in targets {
        world
            .insert_one(*entity, Discarded)
            .map_err(|_| TransitionError::InvalidEntity)?;
    }
    Ok(())
}
