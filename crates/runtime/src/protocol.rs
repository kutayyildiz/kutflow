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
