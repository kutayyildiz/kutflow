use std::collections::BTreeMap;

use hecs::Entity;
use kutflow_core::{Transition, queries::Query, views::View};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Source {
    pub query: Query,
    pub view: View,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChoiceKind {
    Wish,
    Commit,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChoiceEffect {
    #[default]
    Transition,
    Delegate,
    Next,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Choice {
    pub kind: ChoiceKind,
    #[serde(default)]
    pub effect: ChoiceEffect,
    pub transition: Option<Transition>,
    pub because: String,
    #[serde(default)]
    pub next: Option<String>,
    #[serde(default)]
    pub actor_question: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QuestionType {
    Choices,
    Actors,
    Wishes,
    Commits,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StageQuestion {
    pub name: String,
    pub prompt: String,
    #[serde(rename = "type")]
    pub kind: QuestionType,
    #[serde(default)]
    pub source: Option<Source>,
    #[serde(default)]
    pub transitions: Vec<Transition>,
    #[serde(default)]
    pub effects: Vec<ChoiceEffect>,
    #[serde(default = "required")]
    pub required: bool,
}

fn required() -> bool {
    true
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StageEffectKind {
    Apply,
    Generate,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StageEffect {
    pub question: String,
    pub action: StageEffectKind,
    #[serde(default)]
    pub next: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Stage {
    pub name: String,
    pub order: u32,
    #[serde(default)]
    pub entry: bool,
    #[serde(default)]
    pub target: Option<Source>,
    #[serde(default)]
    pub context: Vec<Source>,
    pub actor: String,
    #[serde(default = "default_handoffs")]
    pub max_handoffs: u8,
    pub question: Vec<StageQuestion>,
    pub effect: Vec<StageEffect>,
}

fn default_handoffs() -> u8 {
    3
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActorConfig {
    pub adapter: String,
    pub description: String,
    pub model: Option<String>,
    #[serde(default)]
    pub settings: BTreeMap<String, toml::Value>,
}

#[derive(Debug, Clone)]
pub struct HostConfig {
    pub actors: BTreeMap<Entity, ActorSettings>,
}

#[derive(Debug, Clone)]
pub struct ActorSettings {
    pub model: Option<String>,
    pub settings: BTreeMap<String, toml::Value>,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub stage: Vec<Stage>,
    pub choice: Vec<Choice>,
    pub actors: BTreeMap<String, ActorConfig>,
}
