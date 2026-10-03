use hecs::{Entity, World};
use serde::Deserialize;
use std::{error::Error, fmt};

mod actors;
mod branches;
mod commits;
mod wishes;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Query {
    UnreviewedWishes,
    ReviewedWishes,
    WishesNeedingCommits,
    WishTransitionData,
    UnreviewedCommits,
    ReviewedCommits,
    Commits,
    CommitTransitionData,
    BranchCandidates,
    UnreviewedBranches,
    ActiveBranches,
    Branches,
    BranchCommits,
    BranchTransitionData,
    Actors,
}

impl Query {
    pub fn select(self, world: &World, subject: Option<Entity>) -> Result<Vec<Entity>, QueryError> {
        match self {
            Self::Actors => actors::select(world),
            Self::UnreviewedWishes
            | Self::ReviewedWishes
            | Self::WishesNeedingCommits
            | Self::WishTransitionData => wishes::select(self, world),
            Self::UnreviewedCommits
            | Self::ReviewedCommits
            | Self::Commits
            | Self::CommitTransitionData
            | Self::BranchCandidates => commits::select(self, world),
            _ => branches::select(self, world, subject),
        }
    }
}

#[derive(Debug)]
pub enum QueryError {
    SubjectRequired(Query),
    InvalidSubject,
}

impl fmt::Display for QueryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SubjectRequired(query) => write!(f, "{query:?} requires a subject"),
            Self::InvalidSubject => f.write_str("invalid query subject"),
        }
    }
}
impl Error for QueryError {}
