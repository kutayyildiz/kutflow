use std::collections::BTreeMap;

use crate::{Adapter, AdapterError};
use kutflow_runtime::{Request, RequestKind, Response};

mod options;
mod wire;
use options::{build_options, validate_answer};
pub use wire::{
    Answer, AnswerType, Question, QuestionType, SystemOneRequest, SystemOneResult, SystemOneUsage,
};

const QUESTION_ID: &str = "decision";

pub struct SystemOneAdapter {
    model: String,
}

impl SystemOneAdapter {
    pub fn new(model: impl Into<String>) -> Self {
        Self {
            model: model.into(),
        }
    }
}

impl Adapter for SystemOneAdapter {
    type Request = SystemOneRequest;
    type Response = SystemOneResult;

    fn encode(&self, request: &Request) -> Result<Self::Request, AdapterError> {
        if self.model.trim().is_empty() {
            return Err(AdapterError::InvalidResponse(
                "System One model must not be blank".into(),
            ));
        }
        let RequestKind::Decision { targets, .. } = &request.kind else {
            return Err(AdapterError::Unsupported);
        };
        let options = build_options(request)?;
        let criteria = options
            .iter()
            .map(|(key, description, _)| (key.clone(), description.clone()))
            .collect();
        let target_ids = targets
            .iter()
            .map(|target| target.to_bits().get().to_string())
            .collect::<Vec<_>>()
            .join(", ");
        let question = Question {
            question_type: QuestionType::Choice,
            instructions: format!("{} Target IDs: {target_ids}.", request.prompt),
            criteria,
        };
        Ok(SystemOneRequest {
            model: self.model.clone(),
            state: request.context.clone(),
            questions: BTreeMap::from([(QUESTION_ID.into(), question)]),
        })
    }

    fn decode(
        &self,
        request: &Request,
        response: Self::Response,
    ) -> Result<Response, AdapterError> {
        if response.answers.len() != 1 {
            return Err(AdapterError::InvalidResponse(
                "expected exactly one answer".into(),
            ));
        }
        let answer = response
            .answers
            .get(QUESTION_ID)
            .ok_or_else(|| AdapterError::InvalidResponse("missing decision answer".into()))?;
        if answer.answer_type != AnswerType::Choice {
            return Err(AdapterError::InvalidResponse(
                "answer type must be choice".into(),
            ));
        }
        let options = build_options(request)?;
        validate_answer(
            &options,
            &answer.choice,
            answer.confidence,
            &answer.probabilities,
        )?;
        let decisions = options
            .into_iter()
            .find(|(key, _, _)| key == &answer.choice)
            .map(|(_, _, decisions)| decisions)
            .ok_or_else(|| {
                AdapterError::InvalidResponse("selected choice is not available".into())
            })?;
        let response = Response::Decisions(decisions);
        if !request.accepts(&response) {
            return Err(AdapterError::InvalidResponse(
                "response does not match request".into(),
            ));
        }
        Ok(response)
    }
}
