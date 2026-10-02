use crate::{
    components::{
        BelongsToBranch, Branch, Commit, Data, TransitionDecision, TransitionReason, Transitioned,
    },
    queries::{Query, QueryError},
};
use hecs::{Entity, World};

pub(super) fn select(
    query: Query,
    world: &World,
    subject: Option<Entity>,
) -> Result<Vec<Entity>, QueryError> {
    match query {
        Query::UnreviewedBranches => Ok(world
            .query::<Entity>()
            .with::<&Branch>()
            .without::<&Data>()
            .without::<&TransitionDecision>()
            .iter()
            .collect()),
        Query::ActiveBranches => Ok(world
            .query::<(Entity, &TransitionDecision)>()
            .with::<&Branch>()
            .with::<&Transitioned>()
            .without::<&Data>()
            .iter()
            .filter_map(|(entity, decision)| {
                (decision.0 == crate::Transition::Review).then_some(entity)
            })
            .collect()),
        Query::Branches => Ok(world
            .query::<Entity>()
            .with::<&Branch>()
            .without::<&Data>()
            .iter()
            .collect()),
        Query::BranchCommits => {
            let branch = subject.ok_or(QueryError::SubjectRequired(query))?;
            world
                .get::<&Branch>(branch)
                .map_err(|_| QueryError::InvalidSubject)?;
            Ok(world
                .query::<(Entity, &BelongsToBranch)>()
                .with::<&Commit>()
                .without::<&Data>()
                .iter()
                .filter_map(|(entity, belongs)| (belongs.0 == branch).then_some(entity))
                .collect())
        }
        Query::BranchTransitionData => Ok(world
            .query::<Entity>()
            .with::<(&Data, &Branch, &TransitionReason, &TransitionDecision)>()
            .iter()
            .collect()),
        _ => unreachable!(),
    }
}
