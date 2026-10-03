use hecs::Entity;
use kutflow_core::Transition;

use crate::protocol::{Answer, Request, RequestKind, Response};

#[derive(Clone)]
pub(super) struct ReasonAction {
    pub reason_id: Entity,
    pub transition: Transition,
    pub reason: String,
}

#[derive(Clone)]
pub(super) struct ResolvedAction {
    pub transition: Transition,
    pub target: Entity,
    pub reason: String,
}

pub(super) struct Pending {
    pub request: Request,
    pub action: PendingAction,
    pub response: Option<Response>,
}

#[derive(Clone)]
pub(super) enum PendingAction {
    Answers(Vec<ReasonAction>),
    Generation {
        transition: Transition,
        target: Entity,
    },
}

impl Pending {
    pub fn target(&self) -> Option<Entity> {
        match &self.action {
            PendingAction::Answers(_) => match &self.request.kind {
                RequestKind::Reasons { target, .. } => Some(*target),
                _ => None,
            },
            PendingAction::Generation { target, .. } => Some(*target),
        }
    }
}

pub(super) fn resolve(
    reasons: &[ReasonAction],
    answers: &[Answer],
) -> Result<Vec<ResolvedAction>, ()> {
    let resolved = answers
        .iter()
        .map(|answer| {
            let reason = reasons
                .iter()
                .find(|reason| reason.reason_id == answer.reason_id)
                .ok_or(())?;
            Ok(ResolvedAction {
                transition: reason.transition,
                target: answer.target,
                reason: reason.reason.clone(),
            })
        })
        .collect::<Result<Vec<_>, ()>>()?;
    validate_conflicts(&resolved)?;
    Ok(resolved)
}

fn validate_conflicts(actions: &[ResolvedAction]) -> Result<(), ()> {
    for (index, action) in actions.iter().enumerate() {
        let selected = actions[..=index]
            .iter()
            .filter(|item| item.target == action.target)
            .collect::<Vec<_>>();
        if selected.len() > 2 {
            return Err(());
        }
        if let [first, second] = selected.as_slice()
            && !matches!(
                (first.transition, second.transition),
                (
                    Transition::DiscardWish | Transition::DiscardCommit,
                    Transition::AddWish | Transition::AddCommit
                )
            )
        {
            return Err(());
        }
    }
    Ok(())
}
