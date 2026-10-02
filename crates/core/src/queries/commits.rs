use crate::{
    components::{
        BelongsToBranch, Commit, CommitHash, Data, TransitionDecision, TransitionReason,
        Transitioned,
    },
    queries::{Query, QueryError},
};
use hecs::{Entity, World};

pub(super) fn select(query: Query, world: &World) -> Result<Vec<Entity>, QueryError> {
    match query {
        Query::UnreviewedCommits => Ok(world
            .query::<Entity>()
            .with::<&Commit>()
            .without::<&Data>()
            .without::<&TransitionDecision>()
            .iter()
            .collect()),
        Query::ReviewedCommits => Ok(world
            .query::<(Entity, &TransitionDecision)>()
            .with::<&Commit>()
            .with::<&Transitioned>()
            .without::<&Data>()
            .iter()
            .filter_map(|(entity, decision)| {
                (decision.0 == crate::Transition::Review).then_some(entity)
            })
            .collect()),
        Query::Commits => Ok(world
            .query::<Entity>()
            .with::<&Commit>()
            .without::<&Data>()
            .iter()
            .collect()),
        Query::CommitTransitionData => Ok(world
            .query::<Entity>()
            .with::<(&Data, &Commit, &TransitionReason, &TransitionDecision)>()
            .iter()
            .collect()),
        Query::BranchCandidates => Ok(Query::ReviewedCommits
            .select(world, None)?
            .into_iter()
            .filter(|entity| {
                world.get::<&BelongsToBranch>(*entity).is_err()
                    && world.get::<&CommitHash>(*entity).is_err()
            })
            .collect()),
        _ => unreachable!(),
    }
}
