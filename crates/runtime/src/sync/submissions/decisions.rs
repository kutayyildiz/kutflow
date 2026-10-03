use crate::{
    request::ResolvedAction,
    runtime::{Runtime, RuntimeError},
};
use hecs::Entity;
mod execute;
mod plan;
mod validation;

enum Planned {
    Transition(ResolvedAction),
    Mark(ResolvedAction),
    Next {
        stage: String,
        actor: Option<Entity>,
        reason: Option<String>,
    },
    Delegate {
        actor: Entity,
        reason: String,
    },
    Generate {
        items: Vec<String>,
        wishes: bool,
        next: Option<String>,
    },
}

pub(super) fn apply(
    runtime: &mut Runtime,
    pending: &crate::request::Pending,
    response: &crate::protocol::Response,
) -> Result<Option<Entity>, RuntimeError> {
    let plan = plan::build(runtime, pending, response)?;
    validate_plan(runtime, pending, &plan)?;
    execute::apply(runtime, pending, plan.actions)
}

pub(super) fn validate_response(
    runtime: &Runtime,
    pending: &crate::request::Pending,
    response: &crate::protocol::Response,
) -> Result<(), RuntimeError> {
    let plan = plan::build(runtime, pending, response)?;
    validate_plan(runtime, pending, &plan)
}

fn validate_plan(
    runtime: &Runtime,
    pending: &crate::request::Pending,
    plan: &Plan,
) -> Result<(), RuntimeError> {
    validation::validate(runtime, &plan.transitions)?;
    execute::validate(runtime, pending, &plan.actions)
}

struct Plan {
    actions: Vec<Planned>,
    transitions: Vec<ResolvedAction>,
}
