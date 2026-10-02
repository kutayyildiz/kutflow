#[derive(Debug, Clone)]
pub struct Request {
    pub id: RequestId,
    pub context: String,
    pub prompt: String,
    pub output: OutputKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RequestId(pub u64);

#[derive(Debug, Clone)]
pub enum OutputKind {
    Text,
    List,
    Choice(Vec<String>),
    Probability,
    Score(Vec<String>),
}

#[derive(Debug, Clone)]
pub enum Response {
    Text(String),
    List(Vec<String>),
    Choice(String),
    Probability(f64),
    Score(f64),
}

impl Request {
    pub(crate) fn accepts(&self, response: &Response) -> bool {
        match (&self.output, response) {
            (OutputKind::Text, Response::Text(_)) | (OutputKind::List, Response::List(_)) => true,
            (OutputKind::Choice(choices), Response::Choice(value)) => choices.contains(value),
            (OutputKind::Probability, Response::Probability(value))
            | (OutputKind::Score(_), Response::Score(value)) => (0.0..=1.0).contains(value),
            _ => false,
        }
    }
}
