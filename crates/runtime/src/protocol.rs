use std::collections::BTreeMap;

use hecs::Entity;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct State {
    pub target: String,
    pub context: Vec<Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct Request {
    pub id: RequestId,
    pub target: Entity,
    pub actor: Entity,
    pub stage: String,
    pub state: State,
    pub questions: BTreeMap<String, Question>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RequestId(pub u64);

#[derive(Debug, Clone)]
pub struct Question {
    pub prompt: String,
    pub kind: QuestionKind,
    pub required: bool,
}

#[derive(Debug, Clone)]
pub enum QuestionKind {
    Choice(Vec<OptionValue>),
    Wishes,
    Commits,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OptionValue {
    pub entity: Entity,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Answer {
    Choice(Entity),
    Texts(Vec<String>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
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
