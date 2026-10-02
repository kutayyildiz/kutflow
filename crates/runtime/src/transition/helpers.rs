use hecs::Entity;

use super::TransitionError;
use crate::Response;

pub(super) fn one(targets: &[Entity]) -> Result<Entity, TransitionError> {
    match targets {
        [entity] => Ok(*entity),
        _ => Err(TransitionError::ExpectedOneTarget),
    }
}

pub(super) fn text(response: Option<Response>) -> Result<String, TransitionError> {
    match response {
        Some(Response::Text(value)) => Ok(value),
        _ => Err(TransitionError::ExpectedText),
    }
}

pub(super) fn list(response: Option<Response>) -> Result<Vec<String>, TransitionError> {
    match response {
        Some(Response::List(values)) => Ok(values),
        _ => Err(TransitionError::ExpectedList),
    }
}
