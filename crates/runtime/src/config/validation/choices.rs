use kutflow_core::Transition;

use super::super::loader::ConfigError;
use super::invalid;
use crate::config::{ChoiceEffect, ChoiceKind, Config};

pub(super) fn validate(config: &Config) -> Result<(), ConfigError> {
    for choice in &config.choice {
        if choice.because.trim().is_empty() {
            return invalid("choice because clause must not be blank");
        }
        match choice.effect {
            ChoiceEffect::Transition
                if choice.transition.is_none()
                    || choice.next.is_some()
                    || choice.actor_question.is_some()
                    || matches!(
                        choice.transition,
                        Some(
                            Transition::AddWish | Transition::AddCommit | Transition::CreateBranch
                        )
                    ) =>
            {
                return invalid("transition choice needs a supported transition");
            }
            ChoiceEffect::Delegate
                if choice.transition.is_some()
                    || choice.next.is_some()
                    || choice.actor_question.as_deref().is_none_or(str::is_empty) =>
            {
                return invalid("delegate choice needs only an actor_question");
            }
            ChoiceEffect::Next if choice.next.as_deref().is_none_or(str::is_empty) => {
                return invalid("next choice needs a next stage");
            }
            _ => {}
        }
        if let Some(transition) = choice.transition
            && !legal_choice(choice.kind, transition)
        {
            return invalid("transition is invalid for choice kind");
        }
    }
    Ok(())
}

fn legal_choice(kind: ChoiceKind, transition: Transition) -> bool {
    match kind {
        ChoiceKind::Wish => matches!(
            transition,
            Transition::Review
                | Transition::DiscardWish
                | Transition::AddWish
                | Transition::AddCommit
        ),
        ChoiceKind::Commit => matches!(
            transition,
            Transition::Review
                | Transition::DiscardCommit
                | Transition::AddWish
                | Transition::AddCommit
        ),
    }
}
