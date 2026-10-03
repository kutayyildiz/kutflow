use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use serde::Deserialize;

use super::{
    Config, HostConfig,
    loader::ConfigError,
    model::{AgentConfig, Choice, Flow, Routing, Rule},
};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Document {
    #[serde(default)]
    imports: Vec<String>,
    #[serde(default)]
    rule: Vec<Rule>,
    #[serde(default)]
    choice: Vec<Choice>,
    #[serde(default)]
    agents: Option<BTreeMap<String, AgentConfig>>,
    #[serde(default)]
    routing: Option<Routing>,
}

#[derive(Default)]
struct Merged {
    rule: Vec<Rule>,
    choice: Vec<Choice>,
    host: Option<(BTreeMap<String, AgentConfig>, Routing)>,
}

pub(super) fn parse(source: &str) -> Result<Config, ConfigError> {
    let document: Document = toml::from_str(source).map_err(ConfigError::Toml)?;
    if !document.imports.is_empty() {
        return Err(ConfigError::Invalid(
            "Config::parse cannot resolve imports; use Config::load with a file path".into(),
        ));
    }
    let host = entry_host(&document, "configuration")?;
    Ok(config(document.rule, document.choice, host))
}

pub(super) fn load(path: impl AsRef<Path>) -> Result<Config, ConfigError> {
    let mut merged = Merged::default();
    visit(path.as_ref(), true, &mut Vec::new(), &mut merged)?;
    let (agents, routing) = merged.host.ok_or_else(|| {
        ConfigError::Invalid("entry config must define agents and routing".into())
    })?;
    Ok(config(merged.rule, merged.choice, (agents, routing)))
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
        let cycle = active
            .iter()
            .chain(std::iter::once(&canonical))
            .map(|path| path.display().to_string())
            .collect::<Vec<_>>()
            .join(" -> ");
        return Err(ConfigError::Invalid(format!(
            "config import cycle: {cycle}"
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
        merged.host = Some(entry_host(&document, &path.display().to_string())?);
    } else if document.agents.is_some() || document.routing.is_some() {
        return Err(ConfigError::Invalid(format!(
            "imported config '{}' cannot define agents or routing",
            path.display()
        )));
    }
    let parent = path.parent().unwrap_or(Path::new("."));
    for import in &document.imports {
        visit(&parent.join(import), false, active, merged)?;
    }
    merged.rule.extend(document.rule);
    merged.choice.extend(document.choice);
    Ok(())
}

fn entry_host(
    document: &Document,
    path: &str,
) -> Result<(BTreeMap<String, AgentConfig>, Routing), ConfigError> {
    let agents = document
        .agents
        .clone()
        .ok_or_else(|| ConfigError::Invalid(format!("'{path}' must define agents")))?;
    let routing = document
        .routing
        .clone()
        .ok_or_else(|| ConfigError::Invalid(format!("'{path}' must define routing")))?;
    Ok((agents, routing))
}

fn config(
    rule: Vec<Rule>,
    choice: Vec<Choice>,
    (agents, routing): (BTreeMap<String, AgentConfig>, Routing),
) -> Config {
    Config {
        flow: Flow { rule },
        choice,
        host: HostConfig { agents, routing },
    }
}
