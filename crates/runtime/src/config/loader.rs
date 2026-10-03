use std::{error::Error, fmt, path::Path};

use hecs::World;
use kutflow_core::Transition;
use kutflow_core::components::{Commit, Data, TransitionDecision, TransitionReason, Wish};

use super::{Choice, ChoiceKind, Config, HostConfig, imports, validation};
use crate::Runtime;

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
        for choice in &self.choice {
            let reason = TransitionReason(reason_sentence(choice)?);
            match choice.kind {
                super::ChoiceKind::Wish => {
                    world.spawn((
                        Data,
                        Wish(String::new()),
                        reason,
                        TransitionDecision(choice.transition),
                    ));
                }
                super::ChoiceKind::Commit => {
                    world.spawn((
                        Data,
                        Commit(String::new()),
                        reason,
                        TransitionDecision(choice.transition),
                    ));
                }
            }
        }
        let host = self.host;
        Ok((Runtime::new(world, self.flow), host))
    }
}

fn reason_sentence(choice: &Choice) -> Result<String, ConfigError> {
    let subject = match (choice.kind, choice.transition) {
        (ChoiceKind::Wish, Transition::Review) => "Accept this wish",
        (ChoiceKind::Commit, Transition::Review) => "Accept this proposal",
        (ChoiceKind::Wish, Transition::DiscardWish) => "Discard this wish",
        (ChoiceKind::Commit, Transition::DiscardCommit) => "Discard this proposal",
        (_, Transition::AddWish) => "Add wishes",
        (_, Transition::AddCommit) => "Add commit proposals",
        _ => {
            return Err(ConfigError::Invalid(
                "choice transition has no reason sentence format".into(),
            ));
        }
    };
    Ok(format!("{subject} because {}", choice.because.trim()))
}

#[derive(Debug)]
pub enum ConfigError {
    Io(std::io::Error),
    Toml(toml::de::Error),
    Invalid(String),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "could not read configuration: {error}"),
            Self::Toml(error) => write!(f, "invalid TOML configuration: {error}"),
            Self::Invalid(message) => f.write_str(message),
        }
    }
}

impl Error for ConfigError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::Toml(error) => Some(error),
            Self::Invalid(_) => None,
        }
    }
}
