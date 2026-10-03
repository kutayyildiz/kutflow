use std::collections::BTreeMap;

use crate::{Adapter, AdapterError};
use kutflow_runtime::{Answer as NativeAnswer, QuestionKind, Request, Response};

mod reasons;
mod wire;
use reasons::{criteria, validate_answer};
pub use wire::{
    Answer, AnswerType, Question, QuestionType, SystemOneRequest, SystemOneResult, SystemOneUsage,
};

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
        let questions = request
            .questions
            .iter()
            .map(|(name, question)| {
                let criteria = criteria(question)?;
                Ok((
                    name.clone(),
                    Question {
                        question_type: QuestionType::Choice,
                        instructions: question.prompt.clone(),
                        criteria,
                    },
                ))
            })
            .collect::<Result<BTreeMap<_, _>, AdapterError>>()?;
        let state = serde_json::to_value(&request.state)
            .map_err(|error| AdapterError::InvalidResponse(error.to_string()))?;
        Ok(SystemOneRequest {
            model: self.model.clone(),
            state,
            questions,
        })
    }

    fn decode(
        &self,
        request: &Request,
        response: Self::Response,
    ) -> Result<Response, AdapterError> {
        if response.answers.len() != request.questions.len() {
            return Err(AdapterError::InvalidResponse(
                "answer count does not match question count".into(),
            ));
        }
        let mut answers = BTreeMap::new();
        for (name, question) in &request.questions {
            let wire = response
                .answers
                .get(name)
                .ok_or_else(|| AdapterError::InvalidResponse(format!("missing answer '{name}'")))?;
            if wire.answer_type != AnswerType::Choice {
                return Err(AdapterError::InvalidResponse(
                    "answer type must be choice".into(),
                ));
            }
            let criteria = criteria(question)?;
            validate_answer(
                &criteria,
                &wire.choice,
                wire.confidence,
                &wire.probabilities,
            )?;
            let selected = match &question.kind {
                QuestionKind::Choice(options) => options
                    .iter()
                    .find(|option| option.entity.to_bits().get().to_string() == wire.choice)
                    .map(|option| option.entity),
                _ => None,
            }
            .ok_or_else(|| {
                AdapterError::InvalidResponse("selected option is not available".into())
            })?;
            answers.insert(name.clone(), NativeAnswer::Choice(selected));
        }
        let response = Response { answers };
        request
            .accepts(&response)
            .then_some(response)
            .ok_or_else(|| AdapterError::InvalidResponse("response does not match request".into()))
    }
}
