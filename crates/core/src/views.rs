use std::{error::Error, fmt};

use hecs::{Entity, World};
use serde::Deserialize;

use crate::components::{BelongsToBranch, Branch, Commit, Discarded, TransitionReason, Wish};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum View {
    Wishes,
    Commits,
    Branches,
    TransitionReasons,
}

impl View {
    pub fn read(self, world: &World, entities: &[Entity]) -> Result<Vec<String>, ViewError> {
        entities
            .iter()
            .copied()
            .map(|entity| self.read_one(world, entity))
            .collect()
    }

    pub fn read_one(self, world: &World, entity: Entity) -> Result<String, ViewError> {
        match self {
            Self::Wishes => world
                .get::<&Wish>(entity)
                .map(|value| value.to_string())
                .map_err(|_| ViewError::MissingComponent { entity, view: self }),

            Self::Commits => world
                .get::<&Commit>(entity)
                .map(|value| value.to_string())
                .map_err(|_| ViewError::MissingComponent { entity, view: self }),

            Self::Branches => {
                world
                    .get::<&Branch>(entity)
                    .map_err(|_| ViewError::MissingComponent { entity, view: self })?;
                let mut commits = world
                    .query::<(Entity, &BelongsToBranch, &Commit)>()
                    .iter()
                    .filter_map(|(commit, belongs, value)| {
                        (belongs.0 == entity && world.get::<&Discarded>(commit).is_err())
                            .then_some((commit.to_bits(), value.to_string()))
                    })
                    .collect::<Vec<_>>();
                commits.sort_by_key(|(id, _)| *id);
                Ok(commits
                    .into_iter()
                    .map(|(_, value)| value)
                    .collect::<Vec<_>>()
                    .join("\n"))
            }

            Self::TransitionReasons => world
                .get::<&TransitionReason>(entity)
                .map(|value| value.to_string())
                .map_err(|_| ViewError::MissingComponent { entity, view: self }),
        }
    }
}

#[derive(Debug)]
pub enum ViewError {
    MissingComponent { entity: Entity, view: View },
}

impl fmt::Display for ViewError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingComponent { entity, view } => {
                write!(f, "entity {entity:?} cannot be read using {view:?}")
            }
        }
    }
}

impl Error for ViewError {}
