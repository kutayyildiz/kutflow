use crate::{
    components::{Data, Discarded, TransitionDecision, TransitionReason, Transitioned, Wish},
    queries::{Query, QueryError},
};
use hecs::{Entity, World};

pub(super) fn select(query: Query, world: &World) -> Result<Vec<Entity>, QueryError> {
    match query {
        Query::UnreviewedWishes => Ok(world
            .query::<Entity>()
            .with::<&Wish>()
            .without::<&Data>()
            .without::<&Discarded>()
            .without::<&TransitionDecision>()
            .iter()
            .collect()),
        Query::ReviewedWishes => Ok(world
            .query::<(Entity, &TransitionDecision)>()
            .with::<&Wish>()
            .with::<&Transitioned>()
            .without::<&Data>()
            .without::<&Discarded>()
            .iter()
            .filter_map(|(entity, decision)| {
                (decision.0 == crate::Transition::Review).then_some(entity)
            })
            .collect()),
        Query::WishesNeedingCommits => {
            if !Query::UnreviewedWishes.select(world, None)?.is_empty()
                || !Query::Commits.select(world, None)?.is_empty()
            {
                return Ok(Vec::new());
            }
            Query::ReviewedWishes.select(world, None)
        }
        Query::WishTransitionData => Ok(world
            .query::<Entity>()
            .with::<(&Data, &Wish, &TransitionReason, &TransitionDecision)>()
            .without::<&Discarded>()
            .iter()
            .collect()),
        _ => unreachable!(),
    }
}
