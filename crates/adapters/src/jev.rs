use kutflow_runtime::{OutputKind, Request, Response};

use crate::{Adapter, AdapterError};

pub struct JevAdapter;

pub enum JevRequest {
    Choice {
        state: String,
        prompt: String,
        choices: Vec<String>,
    },
    Noul {
        state: String,
        prompt: String,
    },
    Score {
        state: String,
        prompt: String,
        scale: Vec<String>,
    },
}

pub enum JevResponse {
    Choice(String),
    Noul(f64),
    Score(f64),
}

impl Adapter for JevAdapter {
    type Request = JevRequest;
    type Response = JevResponse;

    fn encode(&self, request: &Request) -> Result<Self::Request, AdapterError> {
        match &request.output {
            OutputKind::Text => Err(AdapterError::Unsupported),

            OutputKind::List => Err(AdapterError::Unsupported),

            OutputKind::Choice(choices) => Ok(JevRequest::Choice {
                state: request.context.clone(),
                prompt: request.prompt.clone(),
                choices: choices.clone(),
            }),

            OutputKind::Probability => Ok(JevRequest::Noul {
                state: request.context.clone(),
                prompt: request.prompt.clone(),
            }),

            OutputKind::Score(scale) => Ok(JevRequest::Score {
                state: request.context.clone(),
                prompt: request.prompt.clone(),
                scale: scale.clone(),
            }),
        }
    }

    fn decode(
        &self,
        request: &Request,
        response: Self::Response,
    ) -> Result<Response, AdapterError> {
        match (&request.output, response) {
            (OutputKind::Choice(choices), JevResponse::Choice(choice))
                if choices.contains(&choice) =>
            {
                Ok(Response::Choice(choice))
            }

            (OutputKind::Probability, JevResponse::Noul(value)) if (0.0..=1.0).contains(&value) => {
                Ok(Response::Probability(value))
            }

            (OutputKind::Score(_), JevResponse::Score(value)) if (0.0..=1.0).contains(&value) => {
                Ok(Response::Score(value))
            }

            _ => Err(AdapterError::InvalidResponse(
                "response does not match request".into(),
            )),
        }
    }
}
