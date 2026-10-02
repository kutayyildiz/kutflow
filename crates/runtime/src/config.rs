use kutflow_core::{queries::Query, views::View};

use serde::Deserialize;

#[derive(Debug, Deserialize)]
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

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Rule {
    pub order: u32,
    pub target: Source,

    #[serde(default)]
    pub context: Vec<Source>,

    pub choices: Source,
    pub mode: RuleMode,
    pub prompt: String,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuleMode {
    PerEntity,
    Batch,
}
