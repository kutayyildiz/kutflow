use kutflow_runtime::{Answer, Request, RequestKind, Response, State};

use crate::{Adapter, AdapterError};

pub struct UserAdapter;

#[derive(Clone)]
pub struct UserRequest {
    pub state: State,
    pub question: String,
    pub kind: RequestKind,
}

pub enum UserResponse {
    Answers(Vec<Answer>),
    Wishes(Vec<String>),
    Commits(Vec<String>),
}

impl Adapter for UserAdapter {
    type Request = UserRequest;
    type Response = UserResponse;

    fn encode(&self, request: &Request) -> Result<Self::Request, AdapterError> {
        Ok(UserRequest {
            state: request.state.clone(),
            question: request.question.clone(),
            kind: request.kind.clone(),
        })
    }

    fn decode(
        &self,
        request: &Request,
        response: Self::Response,
    ) -> Result<Response, AdapterError> {
        let response = match response {
            UserResponse::Answers(items) => Response::Answers(items),
            UserResponse::Wishes(items) => Response::Wishes(items),
            UserResponse::Commits(items) => Response::Commits(items),
        };
        request
            .accepts(&response)
            .then_some(response)
            .ok_or_else(|| AdapterError::InvalidResponse("response does not match request".into()))
    }
}
