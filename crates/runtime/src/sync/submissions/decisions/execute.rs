use super::Planned;
use crate::{
    request::Pending,
    runtime::{Runtime, RuntimeError},
};
use hecs::Entity;
mod actions;

pub(super) fn validate(
    runtime: &Runtime,
    pending: &Pending,
    plan: &[Planned],
) -> Result<(), RuntimeError> {
    runtime
        .world
        .entity(pending.request.target)
        .map_err(|_| RuntimeError::InvalidResponse)?;
    let delegates = plan
        .iter()
        .filter(|action| matches!(action, Planned::Delegate { .. }))
        .count();
    let nexts = plan
        .iter()
        .filter(|action| matches!(action, Planned::Next { .. }))
        .count();
    if delegates > 1 || (delegates == 1 && plan.len() > 1) || nexts > 1 {
        return Err(RuntimeError::InvalidResponse);
    }
    if delegates == 1 {
        let crate::request::PendingAction::Stage { max_handoffs, .. } = &pending.action;
        if pending.handoffs >= *max_handoffs {
            return Err(RuntimeError::InvalidResponse);
        }
    }
    let discards = plan.iter().any(|action| matches!(action, Planned::Transition(a) if matches!(a.transition, kutflow_core::Transition::DiscardWish | kutflow_core::Transition::DiscardCommit)));
    let generates = plan
        .iter()
        .any(|action| matches!(action, Planned::Generate { .. }));
    if discards && generates {
        return Err(RuntimeError::InvalidResponse);
    }
    Ok(())
}

pub(super) fn apply(
    runtime: &mut Runtime,
    pending: &Pending,
    mut plan: Vec<Planned>,
) -> Result<Option<Entity>, RuntimeError> {
    if plan
        .iter()
        .any(|action| matches!(action, Planned::Delegate { .. }))
    {
        let Planned::Delegate { actor, reason } =
            plan.pop().ok_or(RuntimeError::InvalidResponse)?
        else {
            return Err(RuntimeError::InvalidResponse);
        };
        let name = runtime
            .world
            .get::<&kutflow_core::components::Name>(actor)
            .map_err(|_| RuntimeError::InvalidResponse)?;
        runtime
            .history
            .entry(pending.request.target)
            .or_default()
            .extend([reason, format!("Assigned to actor {}.", name.0)]);
        return Ok(Some(actor));
    }
    actions::apply(runtime, pending.request.target, plan)?;
    Ok(None)
}
