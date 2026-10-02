use hecs::Entity;
use kutflow_core::Transition;

use crate::protocol::{Request, Response};

pub(super) struct Pending {
    pub request: Request,
    pub action: PendingAction,
    pub response: Option<Response>,
}

#[derive(Clone)]
pub(super) enum PendingAction {
    Decisions,
    Generation {
        transition: Transition,
        targets: Vec<Entity>,
    },
}

impl Pending {
    pub fn targets(&self) -> &[Entity] {
        match &self.action {
            PendingAction::Decisions => match &self.request.kind {
                crate::protocol::RequestKind::Decision { targets, .. } => targets,
                _ => &[],
            },
            PendingAction::Generation { targets, .. } => targets,
        }
    }
}
