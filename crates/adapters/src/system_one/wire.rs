use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemOneRequest {
    pub model: String,
    pub state: Value,
    pub questions: BTreeMap<String, Question>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Question {
    #[serde(rename = "type")]
    pub question_type: QuestionType,
    pub instructions: String,
    pub criteria: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QuestionType {
    Choice,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemOneResult {
    pub model: String,
    pub answers: BTreeMap<String, Answer>,
    pub usage: SystemOneUsage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Answer {
    #[serde(rename = "type")]
    pub answer_type: AnswerType,
    pub choice: String,
    pub confidence: f64,
    pub probabilities: BTreeMap<String, f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnswerType {
    Choice,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemOneUsage {
    pub input_tokens: u64,
    pub output_tokens: u64,
}
