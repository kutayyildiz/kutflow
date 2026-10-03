use kutflow_core::{queries::Query, views::View};

use super::super::loader::ConfigError;
use super::invalid;
use crate::config::{ChoiceKind, Config, QuestionType, Stage, StageEffectKind};

mod references;
mod sources;
pub(super) use references::validate_references;
use sources::{source, target_kind};

pub(super) fn validate(config: &Config, stage: &Stage) -> Result<(), ConfigError> {
    if stage.name.trim().is_empty()
        || stage.actor.trim().is_empty()
        || stage.max_handoffs == 0
        || !config.actors.contains_key(&stage.actor)
        || stage.question.is_empty()
        || stage.effect.is_empty()
    {
        return invalid(format!(
            "stage '{}' has missing actor, questions, or effects",
            stage.name
        ));
    }
    if stage.entry && stage.target.is_none() {
        return invalid("entry stage needs a target source");
    }
    if let Some(target) = &stage.target {
        if target_kind(target.query).is_none() {
            return invalid("stage target must select wishes or commits");
        }
        source(target)?;
    }
    for item in &stage.context {
        source(item)?;
    }
    unique(stage.question.iter().map(|question| question.name.as_str()))?;
    for question in &stage.question {
        validate_question(config, stage, question)?;
    }
    unique(stage.effect.iter().map(|effect| effect.question.as_str()))?;
    for effect in &stage.effect {
        let question = stage
            .question
            .iter()
            .find(|q| q.name == effect.question)
            .ok_or_else(|| {
                ConfigError::Invalid(format!(
                    "effect refers to unknown question '{}'",
                    effect.question
                ))
            })?;
        match effect.action {
            StageEffectKind::Apply if question.kind != QuestionType::Choices => {
                return invalid("apply effect needs a choice question");
            }
            StageEffectKind::Generate
                if !matches!(question.kind, QuestionType::Wishes | QuestionType::Commits) =>
            {
                return invalid("generate effect needs a text question");
            }
            _ => {}
        }
        if effect.action == StageEffectKind::Apply && effect.next.is_some() {
            return invalid("apply effect cannot select a next stage");
        }
        if !question.required {
            return invalid("effect questions must be required");
        }
    }
    Ok(())
}

fn validate_question(
    config: &Config,
    stage: &Stage,
    question: &crate::config::StageQuestion,
) -> Result<(), ConfigError> {
    if question.name.trim().is_empty() || question.prompt.trim().is_empty() {
        return invalid("question names and prompts must not be blank");
    }
    match question.kind {
        QuestionType::Choices => {
            let source = question
                .source
                .as_ref()
                .ok_or_else(|| ConfigError::Invalid("choice question needs a source".into()))?;
            if !matches!(
                (source.query, source.view),
                (
                    Query::WishTransitionData | Query::CommitTransitionData,
                    View::TransitionReasons
                )
            ) {
                return invalid("choice question source must match its target kind");
            }
            let kind = if source.query == Query::WishTransitionData {
                ChoiceKind::Wish
            } else {
                ChoiceKind::Commit
            };
            if stage
                .target
                .as_ref()
                .and_then(|source| target_kind(source.query))
                .is_some_and(|target| target != kind)
            {
                return invalid("choice question options do not match target kind");
            }
            if !config.choice.iter().any(|choice| choice.kind == kind) {
                return invalid("stage has no options of the matching kind");
            }
            if !config.choice.iter().any(|choice| {
                choice.kind == kind
                    && ((question.transitions.is_empty() && question.effects.is_empty())
                        || choice
                            .transition
                            .is_some_and(|action| question.transitions.contains(&action))
                        || question.effects.contains(&choice.effect))
            }) {
                return invalid("choice filters leave the question without options");
            }
        }
        QuestionType::Actors if question.source.is_some() => {
            return invalid("actor questions cannot have a source");
        }
        QuestionType::Wishes | QuestionType::Commits if question.source.is_some() => {
            return invalid("generation questions cannot have a source");
        }
        _ => {}
    }
    Ok(())
}

fn unique<'a>(values: impl Iterator<Item = &'a str>) -> Result<(), ConfigError> {
    let mut found = std::collections::BTreeSet::new();
    if values
        .into_iter()
        .any(|value| !found.insert(value.to_owned()))
    {
        return invalid("stage names, orders, and question names must be unique");
    }
    Ok(())
}
