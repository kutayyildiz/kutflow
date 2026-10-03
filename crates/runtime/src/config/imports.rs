use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use serde::Deserialize;

use super::{
    Config,
    loader::ConfigError,
    model::{ActorConfig, Choice, Stage},
};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Document {
    #[serde(default)]
    imports: Vec<String>,
    #[serde(default)]
    stage: Vec<Stage>,
    #[serde(default)]
    choice: Vec<Choice>,
    #[serde(default)]
    actors: Option<BTreeMap<String, ActorConfig>>,
}

#[derive(Default)]
struct Merged {
    stage: Vec<Stage>,
    choice: Vec<Choice>,
    actors: Option<BTreeMap<String, ActorConfig>>,
}

pub(super) fn parse(source: &str) -> Result<Config, ConfigError> {
    let document: Document = toml::from_str(source).map_err(ConfigError::Toml)?;
    if !document.imports.is_empty() {
        return Err(ConfigError::Invalid(
            "Config::parse cannot resolve imports; use Config::load with a file path".into(),
        ));
    }
    let actors = document
        .actors
        .ok_or_else(|| ConfigError::Invalid("entry config must define actors".into()))?;
    Ok(Config {
        stage: document.stage,
        choice: document.choice,
        actors,
    })
}

pub(super) fn load(path: impl AsRef<Path>) -> Result<Config, ConfigError> {
    let mut merged = Merged::default();
    visit(path.as_ref(), true, &mut Vec::new(), &mut merged)?;
    Ok(Config {
        stage: merged.stage,
        choice: merged.choice,
        actors: merged
            .actors
            .ok_or_else(|| ConfigError::Invalid("entry config must define actors".into()))?,
    })
}

fn visit(
    path: &Path,
    entry: bool,
    active: &mut Vec<PathBuf>,
    merged: &mut Merged,
) -> Result<(), ConfigError> {
    let canonical = fs::canonicalize(path).map_err(|error| {
        ConfigError::Invalid(format!("cannot resolve '{}': {error}", path.display()))
    })?;
    if active.contains(&canonical) {
        let chain = active
            .iter()
            .chain(std::iter::once(&canonical))
            .map(|item| item.display().to_string())
            .collect::<Vec<_>>()
            .join(" -> ");
        return Err(ConfigError::Invalid(format!(
            "config import cycle: {chain}"
        )));
    }
    active.push(canonical.clone());
    let result = visit_document(&canonical, entry, active, merged);
    active.pop();
    result
}

fn visit_document(
    path: &Path,
    entry: bool,
    active: &mut Vec<PathBuf>,
    merged: &mut Merged,
) -> Result<(), ConfigError> {
    let source = fs::read_to_string(path).map_err(|error| {
        ConfigError::Invalid(format!("cannot read '{}': {error}", path.display()))
    })?;
    let document: Document = toml::from_str(&source).map_err(|error| {
        ConfigError::Invalid(format!("invalid TOML in '{}': {error}", path.display()))
    })?;
    if entry {
        merged.actors = Some(document.actors.ok_or_else(|| {
            ConfigError::Invalid(format!("'{}' must define actors", path.display()))
        })?);
    } else if document.actors.is_some() {
        return Err(ConfigError::Invalid(format!(
            "imported config '{}' cannot define actors",
            path.display()
        )));
    }
    let parent = path.parent().unwrap_or(Path::new("."));
    for import in &document.imports {
        visit(&parent.join(import), false, active, merged)?;
    }
    merged.stage.extend(document.stage);
    merged.choice.extend(document.choice);
    Ok(())
}
