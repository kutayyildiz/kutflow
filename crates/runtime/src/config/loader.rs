use std::path::Path;

use hecs::World;
use kutflow_core::{
    Transition,
    components::{
        Actor, AdapterRef, Commit, Data, Description, Name, TransitionDecision, TransitionReason,
        Wish,
    },
};

use std::collections::BTreeMap;

mod error;
pub use error::ConfigError;

use super::{
    ActorSettings, Choice, ChoiceEffect, ChoiceKind, Config, HostConfig, imports, validation,
};
use crate::Runtime;
use crate::workflow::{ChoiceAction, OptionAction};

impl Config {
    pub fn parse(source: &str) -> Result<Self, ConfigError> {
        let config = imports::parse(source)?;
        validation::validate(&config)?;
        Ok(config)
    }

    pub fn load(path: impl AsRef<Path>) -> Result<Self, ConfigError> {
        let config = imports::load(path)?;
        validation::validate(&config)?;
        Ok(config)
    }

    pub fn into_runtime(self) -> Result<(Runtime, HostConfig), ConfigError> {
        validation::validate(&self)?;
        let mut world = World::new();
        let mut actor_ids = BTreeMap::new();
        let mut settings = BTreeMap::new();
        for (name, actor) in &self.actors {
            let entity = world.spawn((
                Actor,
                Name(name.clone()),
                Description(actor.description.clone()),
                AdapterRef(actor.adapter.clone()),
            ));
            actor_ids.insert(name.clone(), entity);
            settings.insert(
                entity,
                ActorSettings {
                    model: actor.model.clone(),
                    settings: actor.settings.clone(),
                },
            );
        }
        for choice in &self.choice {
            let reason = TransitionReason(reason_sentence(choice)?);
            let action = match choice.effect {
                ChoiceEffect::Transition => {
                    OptionAction::Transition(choice.transition.ok_or_else(|| {
                        ConfigError::Invalid("transition effect needs a transition".into())
                    })?)
                }
                ChoiceEffect::Delegate => OptionAction::Delegate {
                    actor_question: choice.actor_question.clone().ok_or_else(|| {
                        ConfigError::Invalid("delegate effect needs an actor question".into())
                    })?,
                },
                ChoiceEffect::Next => OptionAction::Next {
                    transition: choice.transition,
                    stage: choice
                        .next
                        .clone()
                        .ok_or_else(|| ConfigError::Invalid("next effect needs a stage".into()))?,
                    actor_question: choice.actor_question.clone(),
                },
            };
            match choice.kind {
                super::ChoiceKind::Wish => {
                    let entity =
                        world.spawn((Data, Wish(String::new()), reason, ChoiceAction(action)));
                    seed_transition(&mut world, entity, choice.transition)?;
                }
                super::ChoiceKind::Commit => {
                    let entity =
                        world.spawn((Data, Commit(String::new()), reason, ChoiceAction(action)));
                    seed_transition(&mut world, entity, choice.transition)?;
                }
            }
        }
        let host = HostConfig { actors: settings };
        Ok((Runtime::new(world, self.stage, actor_ids), host))
    }
}

fn reason_sentence(choice: &Choice) -> Result<String, ConfigError> {
    if choice.effect == ChoiceEffect::Delegate {
        return Ok(format!(
            "Ask another actor because {}",
            choice.because.trim()
        ));
    }
    let subject = match (choice.kind, choice.transition) {
        (ChoiceKind::Wish, Some(Transition::Review)) => "Accept this wish",
        (ChoiceKind::Commit, Some(Transition::Review)) => "Accept this proposal",
        (ChoiceKind::Wish, Some(Transition::DiscardWish)) => "Discard this wish",
        (ChoiceKind::Commit, Some(Transition::DiscardCommit)) => "Discard this proposal",
        (_, Some(Transition::AddWish)) => "Add wishes",
        (_, Some(Transition::AddCommit)) => "Add commit proposals",
        (_, None) if choice.effect == ChoiceEffect::Next => "Continue",
        _ => {
            return Err(ConfigError::Invalid(
                "choice transition has no reason sentence format".into(),
            ));
        }
    };
    Ok(format!("{subject} because {}", choice.because.trim()))
}

fn seed_transition(
    world: &mut World,
    entity: hecs::Entity,
    transition: Option<Transition>,
) -> Result<(), ConfigError> {
    if let Some(transition) = transition {
        world
            .insert_one(entity, TransitionDecision(transition))
            .map_err(|_| ConfigError::Invalid("cannot seed choice transition".into()))?;
    }
    Ok(())
}
