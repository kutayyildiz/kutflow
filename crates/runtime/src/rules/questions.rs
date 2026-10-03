use crate::{
    config::{QuestionType, Stage},
    protocol::{OptionValue, Question, QuestionKind},
    runtime::{Runtime, RuntimeError},
};
use hecs::Entity;
use kutflow_core::{queries::Query, views::View};
use std::collections::BTreeMap;

pub(super) fn build(
    runtime: &Runtime,
    target: Entity,
    stage: &Stage,
) -> Result<BTreeMap<String, Question>, RuntimeError> {
    let subject = Some(target);
    let mut questions = BTreeMap::new();
    for config in &stage.question {
        let kind = match config.kind {
            QuestionType::Choices => {
                let source = config.source.as_ref().ok_or(RuntimeError::NoOptions)?;
                QuestionKind::Choice(choice_options(
                    runtime,
                    source.query.select(runtime.world(), subject)?,
                    source.view,
                    config,
                )?)
            }
            QuestionType::Actors => QuestionKind::Choice(actor_options(runtime)?),
            QuestionType::Wishes => QuestionKind::Wishes,
            QuestionType::Commits => QuestionKind::Commits,
        };
        if matches!(&kind, QuestionKind::Choice(values) if values.is_empty()) {
            return Err(RuntimeError::NoOptions);
        }
        questions.insert(
            config.name.clone(),
            Question {
                prompt: config.prompt.clone(),
                kind,
                required: config.required,
            },
        );
    }
    Ok(questions)
}

fn choice_options(
    runtime: &Runtime,
    entities: Vec<Entity>,
    view: View,
    question: &crate::config::StageQuestion,
) -> Result<Vec<OptionValue>, RuntimeError> {
    let mut options = Vec::new();
    for entity in entities {
        let action = runtime
            .world
            .get::<&crate::workflow::ChoiceAction>(entity)
            .map_err(|_| RuntimeError::NoOptions)?;
        let (transition, effect) = match action.0 {
            crate::workflow::OptionAction::Transition(transition) => {
                (Some(transition), crate::config::ChoiceEffect::Transition)
            }
            crate::workflow::OptionAction::Next { transition, .. } => {
                (transition, crate::config::ChoiceEffect::Next)
            }
            crate::workflow::OptionAction::Delegate { .. } => {
                (None, crate::config::ChoiceEffect::Delegate)
            }
        };
        let allowed = (question.transitions.is_empty() && question.effects.is_empty())
            || transition.is_some_and(|action| question.transitions.contains(&action))
            || question.effects.contains(&effect);
        if allowed {
            options.push(OptionValue {
                entity,
                text: view.read_one(runtime.world(), entity)?,
            });
        }
    }
    options.sort_by_key(|option| option.entity.to_bits());
    Ok(options)
}

fn actor_options(runtime: &Runtime) -> Result<Vec<OptionValue>, RuntimeError> {
    Query::Actors
        .select(runtime.world(), None)?
        .into_iter()
        .map(|entity| {
            Ok(OptionValue {
                entity,
                text: View::Actors.read_one(runtime.world(), entity)?,
            })
        })
        .collect()
}
