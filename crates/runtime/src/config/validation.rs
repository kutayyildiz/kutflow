use super::Config;
use super::loader::ConfigError;

mod choices;
mod stages;

pub(super) fn validate(config: &Config) -> Result<(), ConfigError> {
    if config.stage.is_empty() || config.choice.is_empty() || config.actors.is_empty() {
        return invalid("configuration needs actors, stages, and choice options");
    }
    if !config.stage.iter().any(|stage| stage.entry) {
        return invalid("at least one stage must be an entry stage");
    }
    for (name, actor) in &config.actors {
        if name.trim().is_empty()
            || actor.adapter.trim().is_empty()
            || actor.description.trim().is_empty()
            || actor
                .model
                .as_ref()
                .is_some_and(|model| model.trim().is_empty())
        {
            return invalid("actor names, adapters, descriptions, and models must not be blank");
        }
    }
    unique(config.stage.iter().map(|stage| stage.name.as_str()))?;
    let orders = config
        .stage
        .iter()
        .map(|stage| stage.order)
        .collect::<std::collections::BTreeSet<_>>();
    if orders.len() != config.stage.len() {
        return invalid("stage orders must be unique");
    }
    choices::validate(config)?;
    for stage in &config.stage {
        stages::validate(config, stage)?;
    }
    stages::validate_references(config)
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

fn invalid<T>(message: impl Into<String>) -> Result<T, ConfigError> {
    Err(ConfigError::Invalid(message.into()))
}
