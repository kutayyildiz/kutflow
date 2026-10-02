use hecs::Entity;

use crate::{
    protocol::{Request, Response},
    transition::TransitionExt,
};

pub(super) struct Pending {
    pub request: Request,
    pub action: PendingAction,
    pub response: Option<Response>,
}

#[derive(Clone)]
pub(super) enum PendingAction {
    Rule {
        targets: Vec<Entity>,
        choices: Vec<Entity>,
        labels: Vec<String>,
    },
    Transition {
        transition: kutflow_core::Transition,
        targets: Vec<Entity>,
    },
}

impl Pending {
    pub fn apply(
        &self,
        world: &mut hecs::World,
    ) -> Result<Option<(kutflow_core::Transition, Vec<Entity>)>, crate::transition::TransitionError>
    {
        let Some(response) = self.response.clone() else {
            return Ok(None);
        };
        match &self.action {
            PendingAction::Rule {
                targets,
                choices,
                labels,
            } => {
                let Response::Choice(label) = response else {
                    return Err(crate::transition::TransitionError::ExpectedChoice);
                };
                let found = labels
                    .iter()
                    .enumerate()
                    .filter_map(|(index, item)| (item == &label).then_some(choices[index]))
                    .collect::<Vec<_>>();
                let choice = match found.as_slice() {
                    [choice] => *choice,
                    _ => return Err(crate::transition::TransitionError::InvalidChoice),
                };
                let decision = world
                    .get::<&kutflow_core::components::TransitionDecision>(choice)
                    .map_err(|_| {
                        crate::transition::TransitionError::MissingComponent("TransitionDecision")
                    })?
                    .0;
                let reason = world
                    .get::<&kutflow_core::components::TransitionReason>(choice)
                    .map_err(|_| {
                        crate::transition::TransitionError::MissingComponent("TransitionReason")
                    })?
                    .0
                    .clone();
                for target in targets {
                    let _ = world.remove_one::<kutflow_core::components::Transitioned>(*target);
                    world
                        .insert(
                            *target,
                            (
                                kutflow_core::components::TransitionDecision(decision),
                                kutflow_core::components::TransitionReason(reason.clone()),
                            ),
                        )
                        .map_err(|_| crate::transition::TransitionError::InvalidEntity)?;
                }
                return Ok(Some((decision, targets.clone())));
            }
            PendingAction::Transition {
                transition,
                targets,
            } => transition.apply(world, targets, Some(response))?,
        }
        Ok(None)
    }
}
