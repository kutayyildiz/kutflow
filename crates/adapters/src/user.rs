use kutflow_runtime::{OutputKind, Request, Response};

use crate::{Adapter, AdapterError};

pub struct UserAdapter;

pub struct UserRequest {
    pub context: String,
    pub prompt: String,
    pub output: UserOutput,
}

pub enum UserOutput {
    Text,
    List,
    Choice(Vec<String>),
    Probability,
    Score(Vec<String>),
}

pub enum UserResponse {
    Text(String),
    List(Vec<String>),
    Choice(String),
    Probability(f64),
    Score(f64),
}

impl Adapter for UserAdapter {
    type Request = UserRequest;
    type Response = UserResponse;

    fn encode(&self, request: &Request) -> Result<Self::Request, AdapterError> {
        let output = match &request.output {
            OutputKind::Text => UserOutput::Text,
            OutputKind::List => UserOutput::List,
            OutputKind::Choice(choices) => UserOutput::Choice(choices.clone()),
            OutputKind::Probability => UserOutput::Probability,
            OutputKind::Score(scale) => UserOutput::Score(scale.clone()),
        };

        Ok(UserRequest {
            context: request.context.clone(),
            prompt: request.prompt.clone(),
            output,
        })
    }

    fn decode(
        &self,
        request: &Request,
        response: Self::Response,
    ) -> Result<Response, AdapterError> {
        match (&request.output, response) {
            (OutputKind::Text, UserResponse::Text(value)) => Ok(Response::Text(value)),
            (OutputKind::List, UserResponse::List(value)) => Ok(Response::List(value)),

            (OutputKind::Choice(choices), UserResponse::Choice(choice))
                if choices.contains(&choice) =>
            {
                Ok(Response::Choice(choice))
            }

            (OutputKind::Probability, UserResponse::Probability(value))
                if (0.0..=1.0).contains(&value) =>
            {
                Ok(Response::Probability(value))
            }

            (OutputKind::Score(_), UserResponse::Score(value)) if (0.0..=1.0).contains(&value) => {
                Ok(Response::Score(value))
            }

            _ => Err(AdapterError::InvalidResponse(
                "response does not match request".into(),
            )),
        }
    }
}
