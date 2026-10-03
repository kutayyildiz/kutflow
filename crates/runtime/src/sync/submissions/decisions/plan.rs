use super::{Plan, Planned};
use crate::{
    config::StageEffectKind,
    protocol::{Answer, QuestionKind, Response},
    request::{Pending, ResolvedAction},
    runtime::{Runtime, RuntimeError},
    workflow::{ChoiceAction, OptionAction},
};
use hecs::Entity;
use kutflow_core::{Transition, components::TransitionReason};

pub(super) fn build(
    runtime: &Runtime,
    pending: &Pending,
    response: &Response,
) -> Result<Plan, RuntimeError> {
    let mut actions = Vec::new();
    let mut transitions = Vec::new();
    let crate::request::PendingAction::Stage { effects, .. } = &pending.action;
    for effect in effects {
        let answer = response.answers.get(&effect.question);
        if effect.action == StageEffectKind::Apply {
            let Some(Answer::Choice(selected)) = answer else {
                continue;
            };
            let action = runtime
                .world
                .get::<&ChoiceAction>(*selected)
                .map_err(|_| RuntimeError::InvalidResponse)?;
            let reason = runtime
                .world
                .get::<&TransitionReason>(*selected)
                .map_err(|_| RuntimeError::InvalidResponse)?
                .0
                .clone();
            match &action.0 {
                OptionAction::Transition(transition) => {
                    if matches!(transition, Transition::AddWish | Transition::AddCommit) {
                        return Err(RuntimeError::InvalidResponse);
                    }
                    let resolved = ResolvedAction {
                        transition: *transition,
                        target: pending.request.target,
                        reason,
                    };
                    transitions.push(resolved.clone());
                    actions.push(Planned::Transition(resolved));
                }
                OptionAction::Delegate { actor_question } => actions.push(Planned::Delegate {
                    actor: selected_actor(runtime, response, actor_question)?,
                    reason,
                }),
                OptionAction::Next {
                    transition,
                    stage,
                    actor_question,
                } => {
                    let actor = actor_question
                        .as_deref()
                        .map(|name| selected_actor(runtime, response, name))
                        .transpose()?;
                    let next_reason = if let Some(transition) = transition {
                        let resolved = ResolvedAction {
                            transition: *transition,
                            target: pending.request.target,
                            reason,
                        };
                        transitions.push(resolved.clone());
                        actions.push(
                            if matches!(transition, Transition::AddWish | Transition::AddCommit) {
                                Planned::Mark(resolved)
                            } else {
                                Planned::Transition(resolved)
                            },
                        );
                        None
                    } else {
                        Some(reason)
                    };
                    actions.push(Planned::Next {
                        stage: stage.clone(),
                        actor,
                        reason: next_reason,
                    });
                }
            }
        } else if let Some(Answer::Texts(items)) = answer {
            let question = pending
                .request
                .questions
                .get(&effect.question)
                .ok_or(RuntimeError::InvalidResponse)?;
            actions.push(Planned::Generate {
                items: items.clone(),
                wishes: matches!(question.kind, QuestionKind::Wishes),
                next: effect.next.clone(),
            });
        }
    }
    Ok(Plan {
        actions,
        transitions,
    })
}

fn selected_actor(
    runtime: &Runtime,
    response: &Response,
    question: &str,
) -> Result<Entity, RuntimeError> {
    match response.answers.get(question) {
        Some(Answer::Choice(actor))
            if runtime
                .world
                .get::<&kutflow_core::components::Actor>(*actor)
                .is_ok() =>
        {
            Ok(*actor)
        }
        _ => Err(RuntimeError::InvalidResponse),
    }
}
