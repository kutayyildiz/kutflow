use hecs::{Entity, World};

use kutflow_core::{
    Transition,
    components::{Commit, Transitioned, Wish},
};

use crate::{OutputKind, Response};

mod branch;
mod error;
mod helpers;
mod request;
pub use error::TransitionError;
use helpers::{list, one, text};

pub trait TransitionExt {
    fn request(&self) -> Option<(String, OutputKind)>;

    fn apply(
        &self,
        world: &mut World,
        targets: &[Entity],
        response: Option<Response>,
    ) -> Result<(), TransitionError>;
}

impl TransitionExt for Transition {
    fn request(&self) -> Option<(String, OutputKind)> {
        request::for_transition(*self)
    }

    fn apply(
        &self,
        world: &mut World,
        targets: &[Entity],
        response: Option<Response>,
    ) -> Result<(), TransitionError> {
        match self {
            Self::Review => {}

            Self::ModifyWish => {
                let entity = one(targets)?;
                let value = text(response)?;

                world
                    .get::<&mut Wish>(entity)
                    .map_err(|_| TransitionError::MissingComponent("Wish"))?
                    .0 = value;
            }

            Self::DissectWish => {
                for value in list(response)? {
                    world.spawn((Wish(value),));
                }
                for entity in targets {
                    world
                        .despawn(*entity)
                        .map_err(|_| TransitionError::InvalidEntity)?;
                }
            }

            Self::DiscardWish => {
                for entity in targets {
                    world
                        .despawn(*entity)
                        .map_err(|_| TransitionError::InvalidEntity)?;
                }
            }

            Self::AddCommit => {
                for value in list(response)? {
                    world.spawn((Commit(value),));
                }
            }

            Self::ModifyCommit => {
                let entity = one(targets)?;
                let value = text(response)?;

                world
                    .get::<&mut Commit>(entity)
                    .map_err(|_| TransitionError::MissingComponent("Commit"))?
                    .0 = value;
            }

            Self::DissectCommit => {
                for value in list(response)? {
                    world.spawn((Commit(value),));
                }
                for entity in targets {
                    world
                        .despawn(*entity)
                        .map_err(|_| TransitionError::InvalidEntity)?;
                }
            }

            Self::DiscardCommit => {
                for entity in targets {
                    world
                        .despawn(*entity)
                        .map_err(|_| TransitionError::InvalidEntity)?;
                }
            }

            Self::CreateBranch => {
                branch::create(world, targets)?;
            }

            Self::DiscardBranch => {
                branch::discard(world, targets)?;
                let branch = one(targets)?;
                world
                    .despawn(branch)
                    .map_err(|_| TransitionError::InvalidEntity)?;
            }
        }

        for entity in targets {
            if matches!(self, Self::ModifyWish | Self::ModifyCommit) {
                let _ = world.remove_one::<kutflow_core::components::TransitionDecision>(*entity);
                let _ = world.remove_one::<kutflow_core::components::TransitionReason>(*entity);
                let _ = world.remove_one::<Transitioned>(*entity);
            } else if !matches!(
                self,
                Self::DissectWish
                    | Self::DiscardWish
                    | Self::DissectCommit
                    | Self::DiscardCommit
                    | Self::DiscardBranch
            ) {
                world
                    .insert_one(*entity, Transitioned)
                    .map_err(|_| TransitionError::InvalidEntity)?;
            }
        }

        Ok(())
    }
}
