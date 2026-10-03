use std::collections::BTreeMap;

use crate::{Adapter, AdapterError};
use kutflow_runtime::{Answer as NativeAnswer, Request, RequestKind, Response};

mod reasons;
mod wire;
use reasons::{criteria, validate_answer};
pub use wire::{
    Answer, AnswerType, Question, QuestionType, SystemOneRequest, SystemOneResult, SystemOneUsage,
};

const QUESTION_ID: &str = "reason";

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
        let RequestKind::Reasons { .. } = &request.kind else {
            return Err(AdapterError::Unsupported);
        };
        let criteria = criteria(request)?;
        let state = serde_json::to_value(&request.state)
            .map_err(|error| AdapterError::InvalidResponse(error.to_string()))?;
        let question = Question {
            question_type: QuestionType::Choice,
            instructions: request.question.clone(),
            criteria,
        };
        Ok(SystemOneRequest {
            model: self.model.clone(),
            state,
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
            .ok_or_else(|| AdapterError::InvalidResponse("missing reason answer".into()))?;
        if answer.answer_type != AnswerType::Choice {
            return Err(AdapterError::InvalidResponse(
                "answer type must be choice".into(),
            ));
        }
        let criteria = criteria(request)?;
        validate_answer(
            &criteria,
            &answer.choice,
            answer.confidence,
            &answer.probabilities,
        )?;
        let RequestKind::Reasons { target, reasons } = &request.kind else {
            return Err(AdapterError::Unsupported);
        };
        let reason = reasons
            .iter()
            .find(|reason| reason.reason_id.to_bits().get().to_string() == answer.choice)
            .map(|reason| reason.reason_id)
            .ok_or_else(|| {
                AdapterError::InvalidResponse("selected reason is not available".into())
            })?;
        let response = Response::Answers(vec![NativeAnswer {
            reason_id: reason,
            target: *target,
        }]);
        if !request.accepts(&response) {
            return Err(AdapterError::InvalidResponse(
                "response does not match request".into(),
            ));
        }
        Ok(response)
    }
}
