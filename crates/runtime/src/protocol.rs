use hecs::Entity;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct State {
    pub targets: Vec<String>,
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
        targets: Vec<Entity>,
        reasons: Vec<Reason>,
    },
    Wishes,
    Commits,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reason {
    pub entity: Entity,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Answer {
    pub reason: Entity,
    pub targets: Vec<Entity>,
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
            (RequestKind::Reasons { targets, reasons }, Response::Answers(answers)) => {
                valid_answers(targets, reasons, answers)
            }
            _ => false,
        }
    }
}

fn valid_answers(targets: &[Entity], reasons: &[Reason], answers: &[Answer]) -> bool {
    if answers.is_empty() {
        return false;
    }
    for (index, answer) in answers.iter().enumerate() {
        if !reasons.iter().any(|reason| reason.entity == answer.reason) || answer.targets.is_empty()
        {
            return false;
        }
        if answer
            .targets
            .iter()
            .enumerate()
            .any(|(i, target)| answer.targets[..i].contains(target))
        {
            return false;
        }
        for target in &answer.targets {
            if !targets.contains(target)
                || answers[..index]
                    .iter()
                    .any(|prior| prior.reason == answer.reason && prior.targets.contains(target))
            {
                return false;
            }
        }
    }
    true
}
