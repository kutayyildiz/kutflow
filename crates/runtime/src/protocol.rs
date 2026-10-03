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
    pub state: State,
    pub question: String,
    pub kind: RequestKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RequestId(pub u64);

#[derive(Debug, Clone)]
pub enum RequestKind {
    Reasons {
        target: Entity,
        reasons: Vec<Reason>,
    },
    Wishes,
    Commits,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reason {
    pub reason_id: Entity,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Answer {
    pub reason_id: Entity,
    pub target: Entity,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Response {
    Answers(Vec<Answer>),
    Wishes(Vec<String>),
    Commits(Vec<String>),
}

impl Request {
    pub fn accepts(&self, response: &Response) -> bool {
        match (&self.kind, response) {
            (RequestKind::Wishes, Response::Wishes(values))
            | (RequestKind::Commits, Response::Commits(values)) => {
                values.iter().all(|value| !value.trim().is_empty())
            }
            (RequestKind::Reasons { target, reasons }, Response::Answers(answers)) => {
                valid_answers(*target, reasons, answers)
            }
            _ => false,
        }
    }
}

fn valid_answers(target: Entity, reasons: &[Reason], answers: &[Answer]) -> bool {
    if answers.is_empty() {
        return false;
    }
    for (index, answer) in answers.iter().enumerate() {
        if !reasons
            .iter()
            .any(|reason| reason.reason_id == answer.reason_id)
            || answer.target != target
        {
            return false;
        }
        if answers[..index]
            .iter()
            .any(|prior| prior.reason_id == answer.reason_id)
        {
            return false;
        }
    }
    true
}
