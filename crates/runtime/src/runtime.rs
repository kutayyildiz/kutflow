use std::collections::{BTreeMap, HashMap};

use hecs::{Entity, World};
use kutflow_core::components::{Actor, Wish};

use crate::{
    config::Stage,
    protocol::{Question, Request, RequestId, Response, State},
    request::{Pending, PendingAction},
    sync,
};

mod error;
pub use error::RuntimeError;

pub struct Runtime {
    pub(super) world: World,
    pub(super) stages: Vec<Stage>,
    pub(super) actor_ids: BTreeMap<String, Entity>,
    pub(super) stage_for: HashMap<Entity, String>,
    pub(super) actor_for: HashMap<Entity, Entity>,
    pub(super) history: HashMap<Entity, Vec<String>>,
    pub(super) pending: Vec<Pending>,
    pub(super) next_id: u64,
}

impl Runtime {
    pub fn new(world: World, mut stages: Vec<Stage>, actor_ids: BTreeMap<String, Entity>) -> Self {
        stages.sort_by_key(|stage| stage.order);
        Self {
            world,
            stages,
            actor_ids,
            stage_for: HashMap::new(),
            actor_for: HashMap::new(),
            history: HashMap::new(),
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

    pub fn assign_actor(&mut self, id: RequestId, actor: Entity) -> Result<(), RuntimeError> {
        if self.world.get::<&Actor>(actor).is_err() {
            return Err(RuntimeError::InvalidResponse);
        }
        let actor_name = self
            .world
            .get::<&kutflow_core::components::Name>(actor)
            .map(|name| name.0.clone())
            .unwrap_or_else(|_| format!("{actor:?}"));
        let pending = self
            .pending
            .iter_mut()
            .find(|pending| pending.request.id == id)
            .ok_or(RuntimeError::UnknownRequest(id))?;
        if pending.response.is_some() {
            return Err(RuntimeError::InvalidResponse);
        }
        let crate::request::PendingAction::Stage { max_handoffs, .. } = &pending.action;
        if pending.handoffs >= *max_handoffs {
            return Err(RuntimeError::InvalidResponse);
        }
        let note = format!("Assigned to actor {actor_name}.");
        self.history
            .entry(pending.request.target)
            .or_default()
            .push(note.clone());
        pending.request.state.context.push(vec![note]);
        pending.request.actor = actor;
        pending.handoffs += 1;
        Ok(())
    }

    pub fn submit(&mut self, id: RequestId, response: Response) -> Result<(), RuntimeError> {
        let index = self
            .pending
            .iter()
            .position(|pending| pending.request.id == id)
            .ok_or(RuntimeError::UnknownRequest(id))?;
        let pending = self.pending[index].clone();
        if !pending.request.accepts(&response) {
            return Err(RuntimeError::InvalidResponse);
        }
        crate::sync::validate_submission(self, &pending, &response)?;
        self.pending[index].response = Some(response);
        Ok(())
    }

    pub fn world(&self) -> &World {
        &self.world
    }

    pub(super) fn push(
        &mut self,
        target: Entity,
        actor: Entity,
        state: State,
        stage: &Stage,
        questions: BTreeMap<String, Question>,
    ) {
        let id = RequestId(self.next_id);
        self.next_id += 1;
        let request = Request {
            id,
            target,
            actor,
            stage: stage.name.clone(),
            state,
            questions,
        };
        self.pending.push(Pending {
            request,
            action: PendingAction::Stage {
                effects: stage.effect.clone(),
                max_handoffs: stage.max_handoffs,
            },
            response: None,
            handoffs: 0,
        });
    }
}
