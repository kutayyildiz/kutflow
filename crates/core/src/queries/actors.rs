use crate::components::Actor;
use hecs::{Entity, World};

pub(super) fn select(world: &World) -> Result<Vec<Entity>, super::QueryError> {
    let mut actors = world
        .query::<Entity>()
        .with::<&Actor>()
        .iter()
        .collect::<Vec<_>>();
    actors.sort_by_key(|entity| entity.to_bits());
    Ok(actors)
}
