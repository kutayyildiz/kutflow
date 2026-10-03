use hecs::Entity;
use kutflow_core::Transition;

use crate::protocol::{Answer, Request, Response};

#[derive(Clone)]
pub(super) struct ReasonAction {
    pub entity: Entity,
    pub transition: Transition,
    pub reason: String,
}

#[derive(Clone)]
pub(super) struct ResolvedAction {
    pub transition: Transition,
    pub targets: Vec<Entity>,
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
        targets: Vec<Entity>,
    },
}

impl Pending {
    pub fn targets(&self) -> &[Entity] {
        match &self.action {
            PendingAction::Answers(_) => match &self.request.kind {
                crate::protocol::RequestKind::Reasons { targets, .. } => targets,
                _ => &[],
            },
            PendingAction::Generation { targets, .. } => targets,
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
                .find(|reason| reason.entity == answer.reason)
                .ok_or(())?;
            Ok(ResolvedAction {
                transition: reason.transition,
                targets: answer.targets.clone(),
                reason: reason.reason.clone(),
            })
        })
        .collect::<Result<Vec<_>, ()>>()?;
    validate_conflicts(&resolved)?;
    Ok(resolved)
}

fn validate_conflicts(decisions: &[ResolvedAction]) -> Result<(), ()> {
    for (index, decision) in decisions.iter().enumerate() {
        for target in &decision.targets {
            let selected = decisions[..=index]
                .iter()
                .filter(|item| item.targets.contains(target))
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
    }
    Ok(())
}
