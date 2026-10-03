use super::super::super::loader::ConfigError;
use super::super::invalid;
use super::sources::target_kind;
use crate::config::{ChoiceKind, Config, QuestionType, Stage};

pub(in crate::config::validation) fn validate_references(
    config: &Config,
) -> Result<(), ConfigError> {
    for choice in &config.choice {
        if let Some(next) = &choice.next {
            find_stage(config, next)?;
        }
    }
    for stage in &config.stage {
        for choice in &config.choice {
            let available = stage
                .question
                .iter()
                .filter(|q| q.kind == QuestionType::Choices)
                .any(|question| {
                    visible(question, choice)
                        && question.source.as_ref().is_some_and(|source| {
                            source.query
                                == if choice.kind == ChoiceKind::Wish {
                                    kutflow_core::queries::Query::WishTransitionData
                                } else {
                                    kutflow_core::queries::Query::CommitTransitionData
                                }
                        })
                        && stage
                            .target
                            .as_ref()
                            .and_then(|source| target_kind(source.query))
                            .is_none_or(|kind| kind == choice.kind)
                });
            if available {
                validate_choice_stage(config, stage, choice)?;
            }
        }
        for effect in &stage.effect {
            if let Some(next) = &effect.next {
                let next_stage = find_stage(config, next)?;
                let question = stage
                    .question
                    .iter()
                    .find(|q| q.name == effect.question)
                    .expect("validated question");
                let generated_kind = match question.kind {
                    QuestionType::Wishes => Some(ChoiceKind::Wish),
                    QuestionType::Commits => Some(ChoiceKind::Commit),
                    _ => None,
                };
                if next_stage
                    .target
                    .as_ref()
                    .and_then(|target| target_kind(target.query))
                    .is_some_and(|kind| Some(kind) != generated_kind)
                {
                    return invalid("generated item kind does not match next stage target");
                }
            }
        }
    }
    Ok(())
}

fn validate_choice_stage(
    config: &Config,
    stage: &Stage,
    choice: &crate::config::Choice,
) -> Result<(), ConfigError> {
    if let Some(next) = &choice.next {
        let next_stage = find_stage(config, next)?;
        if let (Some(source), Some(target)) = (&stage.target, &next_stage.target)
            && target_kind(source.query).is_some_and(|kind| target_kind(target.query) != Some(kind))
        {
            return invalid("next stage target kind does not match source");
        }
        if let Some(transition) = choice.transition {
            let generated_kind = match transition {
                kutflow_core::Transition::AddWish => Some(QuestionType::Wishes),
                kutflow_core::Transition::AddCommit => Some(QuestionType::Commits),
                _ => None,
            };
            if let Some(kind) = generated_kind
                && !next_stage.question.iter().any(|question| {
                    question.kind == kind
                        && next_stage.effect.iter().any(|effect| {
                            effect.question == question.name
                                && effect.action == crate::config::StageEffectKind::Generate
                        })
                })
            {
                return invalid("add choice next stage must generate the matching item kind");
            }
        }
    }
    if let Some(question) = &choice.actor_question
        && !stage
            .question
            .iter()
            .any(|q| q.name == *question && q.kind == QuestionType::Actors && q.required)
    {
        return invalid(format!(
            "stage '{}' needs required actor question '{question}'",
            stage.name
        ));
    }
    Ok(())
}

fn visible(question: &crate::config::StageQuestion, choice: &crate::config::Choice) -> bool {
    (question.transitions.is_empty() && question.effects.is_empty())
        || choice
            .transition
            .is_some_and(|transition| question.transitions.contains(&transition))
        || question.effects.contains(&choice.effect)
}

fn find_stage<'a>(config: &'a Config, name: &str) -> Result<&'a Stage, ConfigError> {
    config
        .stage
        .iter()
        .find(|stage| stage.name == name)
        .ok_or_else(|| ConfigError::Invalid(format!("unknown stage '{name}'")))
}
