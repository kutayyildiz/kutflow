use std::collections::BTreeMap;

use kutflow_core::{Transition, queries::Query, views::View};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Flow {
    pub rule: Vec<Rule>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Source {
    pub query: Query,
    pub view: View,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Rule {
    pub order: u32,
    pub target: Source,
    #[serde(default)]
    pub context: Vec<Source>,
    pub choices: Source,
    pub prompt: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChoiceKind {
    Wish,
    Commit,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Choice {
    pub kind: ChoiceKind,
    pub transition: Transition,
    pub because: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AgentConfig {
    pub adapter: String,
    #[serde(default)]
    pub model: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Routing {
    pub decision: String,
    pub add_wishes: String,
    pub add_commits: String,
}

#[derive(Debug, Clone)]
pub struct HostConfig {
    pub agents: BTreeMap<String, AgentConfig>,
    pub routing: Routing,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub flow: Flow,
    pub choice: Vec<Choice>,
    pub host: HostConfig,
}
