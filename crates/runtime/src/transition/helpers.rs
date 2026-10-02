use hecs::Entity;

use super::TransitionError;

pub(super) fn one(targets: &[Entity]) -> Result<Entity, TransitionError> {
    match targets {
        [entity] => Ok(*entity),
        _ => Err(TransitionError::ExpectedOneTarget),
    }
}
