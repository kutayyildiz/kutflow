use std::collections::BTreeMap;

use kutflow_runtime::{Answer, QuestionKind, Request, Response};
use serde::Deserialize;

use crate::{Adapter, AdapterError};

pub struct LlmAdapter;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AnswerResponse {
    answers: BTreeMap<String, AnswerValue>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AnswerValue {
    choice: Option<u64>,
    texts: Option<Vec<String>>,
}

impl Adapter for LlmAdapter {
    type Request = String;
    type Response = String;

    fn encode(&self, request: &Request) -> Result<String, AdapterError> {
        let state = serde_json::to_string_pretty(&request.state).map_err(invalid)?;
        let mut prompt = format!(
            "State:\n{state}\n\nReturn one JSON object shaped like {{\"answers\":{{\"question_name\":{{\"choice\":entity_id}}}}}}. Text questions use {{\"texts\":[\"...\"]}}.\nQuestions:"
        );
        for (name, question) in &request.questions {
            prompt.push_str(&format!("\n{name}: {}", question.prompt));
            match &question.kind {
                QuestionKind::Choice(options) => {
                    prompt.push_str("\nChoose one option by its numeric entity:");
                    for option in options {
                        prompt.push_str(&format!(
                            "\n- {}: {}",
                            option.entity.to_bits().get(),
                            option.text
                        ));
                    }
                }
                QuestionKind::Wishes | QuestionKind::Commits => {
                    prompt.push_str("\nProvide generated strings in the texts array.")
                }
            }
        }
        Ok(prompt)
    }

    fn decode(&self, request: &Request, response: String) -> Result<Response, AdapterError> {
        let parsed: AnswerResponse = serde_json::from_str(response.trim()).map_err(invalid)?;
        let mut answers = BTreeMap::new();
        for (name, value) in parsed.answers {
            let question = request.questions.get(&name).ok_or_else(|| {
                AdapterError::InvalidResponse(format!("unknown question '{name}'"))
            })?;
            let answer = match &question.kind {
                QuestionKind::Choice(options) => {
                    if value.texts.is_some() {
                        return Err(AdapterError::InvalidResponse(
                            "choice answer cannot include texts".into(),
                        ));
                    }
                    let id = value.choice.ok_or_else(|| {
                        AdapterError::InvalidResponse("choice answer needs an entity".into())
                    })?;
                    let entity = options
                        .iter()
                        .find(|option| option.entity.to_bits().get() == id)
                        .map(|option| option.entity)
                        .ok_or_else(|| {
                            AdapterError::InvalidResponse("unknown option entity".into())
                        })?;
                    Answer::Choice(entity)
                }
                QuestionKind::Wishes | QuestionKind::Commits => {
                    if value.choice.is_some() {
                        return Err(AdapterError::InvalidResponse(
                            "text answer cannot include a choice".into(),
                        ));
                    }
                    Answer::Texts(value.texts.ok_or_else(|| {
                        AdapterError::InvalidResponse("text answer needs texts".into())
                    })?)
                }
            };
            answers.insert(name, answer);
        }
        let result = Response { answers };
        request
            .accepts(&result)
            .then_some(result)
            .ok_or_else(|| AdapterError::InvalidResponse("response does not match request".into()))
    }
}

fn invalid(error: impl std::fmt::Display) -> AdapterError {
    AdapterError::InvalidResponse(error.to_string())
}
