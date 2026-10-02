use std::{error::Error, fmt};

use hecs::{Entity, World};
use kutflow_core::{components::Wish, queries::QueryError, views::ViewError};

use crate::{
    config::Flow,
    protocol::{Request, RequestId, Response},
    request::{Pending, PendingAction},
    sync,
};

pub struct Runtime {
    pub(super) world: World,
    pub(super) flow: Flow,
    pub(super) pending: Vec<Pending>,
    pub(super) next_id: u64,
}

impl Runtime {
    pub fn new(world: World, flow: Flow) -> Self {
        Self {
            world,
            flow,
            pending: Vec::new(),
            next_id: 1,
        }
    }

    pub fn add_wish(&mut self, wish: impl Into<String>) -> Entity {
        self.world.spawn((Wish(wish.into()),))
    }

    pub fn sync(&mut self) -> Result<(), RuntimeError> {
        sync::sync(self)
    }

    pub fn requests(&self) -> Vec<Request> {
        self.pending
            .iter()
            .filter(|pending| pending.response.is_none())
            .map(|pending| pending.request.clone())
            .collect()
    }

    pub fn submit(&mut self, id: RequestId, response: Response) -> Result<(), RuntimeError> {
        let pending = self
            .pending
            .iter_mut()
            .find(|pending| pending.request.id == id)
            .ok_or(RuntimeError::UnknownRequest(id))?;
        if !pending.request.accepts(&response) {
            return Err(RuntimeError::InvalidResponse);
        }
        pending.response = Some(response);
        Ok(())
    }

    pub fn world(&self) -> &World {
        &self.world
    }
    pub(super) fn push(&mut self, request: Request, action: PendingAction) {
        self.pending.push(Pending {
            request,
            action,
            response: None,
        });
    }

    pub(super) fn request(
        &mut self,
        context: String,
        prompt: String,
        output: crate::OutputKind,
        action: PendingAction,
    ) {
        let id = RequestId(self.next_id);
        self.next_id += 1;
        self.push(
            Request {
                id,
                context,
                prompt,
                output,
            },
            action,
        );
    }
}

#[derive(Debug)]
pub enum RuntimeError {
    Query(QueryError),
    View(ViewError),
    NoChoices,
    AmbiguousChoice(String),
    UnknownRequest(RequestId),
    InvalidResponse,
    Transition(crate::transition::TransitionError),
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
impl From<crate::transition::TransitionError> for RuntimeError {
    fn from(error: crate::transition::TransitionError) -> Self {
        Self::Transition(error)
    }
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Query(error) => error.fmt(f),
            Self::View(error) => error.fmt(f),
            Self::NoChoices => f.write_str("applicable rule produced no choices"),
            Self::AmbiguousChoice(value) => write!(f, "choice label is ambiguous: {value}"),
            Self::UnknownRequest(id) => write!(f, "unknown request {}", id.0),
            Self::InvalidResponse => f.write_str("response does not match request"),
            Self::Transition(error) => error.fmt(f),
        }
    }
}
impl Error for RuntimeError {}
