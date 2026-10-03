use super::super::super::loader::ConfigError;
use super::super::invalid;
use crate::config::{ChoiceKind, Source};
use kutflow_core::{queries::Query, views::View};

pub(super) fn source(source: &Source) -> Result<(), ConfigError> {
    let expected = match source.query {
        Query::UnreviewedWishes | Query::ReviewedWishes | Query::WishesNeedingCommits => {
            View::Wishes
        }
        Query::UnreviewedCommits
        | Query::ReviewedCommits
        | Query::Commits
        | Query::BranchCandidates => View::Commits,
        Query::UnreviewedBranches | Query::ActiveBranches | Query::Branches => View::Branches,
        Query::WishTransitionData | Query::CommitTransitionData | Query::BranchTransitionData => {
            View::TransitionReasons
        }
        Query::Actors => View::Actors,
        Query::BranchCommits => return invalid("branch_commits source is not supported"),
    };
    if source.view != expected {
        return invalid("query and view do not match");
    }
    Ok(())
}

pub(super) fn target_kind(query: Query) -> Option<ChoiceKind> {
    match query {
        Query::UnreviewedWishes | Query::ReviewedWishes | Query::WishesNeedingCommits => {
            Some(ChoiceKind::Wish)
        }
        Query::UnreviewedCommits
        | Query::ReviewedCommits
        | Query::Commits
        | Query::BranchCandidates => Some(ChoiceKind::Commit),
        _ => None,
    }
}
