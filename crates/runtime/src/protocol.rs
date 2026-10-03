use std::collections::BTreeMap;

use hecs::Entity;
use serde::{Deserialize, Serialize};

use kutflow_core::entity;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct State {
    pub target: String,
    pub context: Vec<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub id: RequestId,
    #[serde(with = "entity")]
    pub target: Entity,
    #[serde(with = "entity")]
    pub actor: Entity,
    pub stage: String,
    pub state: State,
    pub questions: BTreeMap<String, Question>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RequestId(pub u64);

impl Serialize for RequestId {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0.to_string())
    }
}

impl<'de> Deserialize<'de> for RequestId {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let id = String::deserialize(deserializer)?;
        id.parse().map(Self).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Question {
    pub prompt: String,
    pub kind: QuestionKind,
    pub required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(
    tag = "type",
    content = "value",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum QuestionKind {
    Choice(Vec<OptionValue>),
    Wishes,
    Commits,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OptionValue {
    #[serde(with = "entity")]
    pub entity: Entity,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "type",
    content = "value",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum Answer {
    #[serde(with = "entity")]
    Choice(Entity),
    Texts(Vec<String>),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Response {
    pub answers: BTreeMap<String, Answer>,
}

impl Request {
    pub fn accepts(&self, response: &Response) -> bool {
        !response.answers.is_empty()
            && response.answers.iter().all(|(name, answer)| {
                self.questions
                    .get(name)
                    .is_some_and(|question| match (&question.kind, answer) {
                        (QuestionKind::Choice(options), Answer::Choice(selected)) => {
                            options.iter().any(|option| option.entity == *selected)
                        }
                        (QuestionKind::Wishes | QuestionKind::Commits, Answer::Texts(items)) => {
                            items.iter().all(|item| !item.trim().is_empty())
                        }
                        _ => false,
                    })
            })
            && self
                .questions
                .iter()
                .all(|(name, question)| !question.required || response.answers.contains_key(name))
    }
}
