use hecs::Entity;
use kutflow_core::Transition;

use crate::{
    config::StageEffect,
    protocol::{Request, Response},
};

#[derive(Clone)]
pub(super) struct ResolvedAction {
    pub transition: Transition,
    pub target: Entity,
    pub reason: String,
}

#[derive(Clone)]
pub(super) struct Pending {
    pub request: Request,
    pub action: PendingAction,
    pub response: Option<Response>,
    pub handoffs: u8,
}

#[derive(Clone)]
pub(super) enum PendingAction {
    Stage {
        effects: Vec<StageEffect>,
        max_handoffs: u8,
    },
}
