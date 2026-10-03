use crate::{protocol::RequestId, transition::TransitionError};
use kutflow_core::{queries::QueryError, views::ViewError};
use std::{error::Error, fmt};
#[derive(Debug)]
pub enum RuntimeError {
    Query(QueryError),
    View(ViewError),
    NoOptions,
    UnknownRequest(RequestId),
    InvalidResponse,
    Transition(TransitionError),
}
impl From<QueryError> for RuntimeError {
    fn from(error: QueryError) -> Self {
        Self::Query(error)
    }
}
impl From<ViewError> for RuntimeError {
    fn from(error: ViewError) -> Self {
        Self::View(error)
    }
}
impl From<TransitionError> for RuntimeError {
    fn from(error: TransitionError) -> Self {
        Self::Transition(error)
    }
}
impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Query(error) => error.fmt(f),
            Self::View(error) => error.fmt(f),
            Self::NoOptions => f.write_str("stage produced no available options"),
            Self::UnknownRequest(id) => write!(f, "unknown request {}", id.0),
            Self::InvalidResponse => f.write_str("response does not match request"),
            Self::Transition(error) => error.fmt(f),
        }
    }
}
impl Error for RuntimeError {}
