use kutflow_runtime::{Decision, Request, RequestKind, Response};

use crate::{Adapter, AdapterError};

pub struct UserAdapter;

#[derive(Clone)]
pub struct UserRequest {
    pub context: String,
    pub prompt: String,
    pub kind: RequestKind,
}

pub enum UserResponse {
    Decisions(Vec<Decision>),
    Wishes(Vec<String>),
    Commits(Vec<String>),
}

impl Adapter for UserAdapter {
    type Request = UserRequest;
    type Response = UserResponse;

    fn encode(&self, request: &Request) -> Result<Self::Request, AdapterError> {
        Ok(UserRequest {
            context: request.context.clone(),
            prompt: request.prompt.clone(),
            kind: request.kind.clone(),
        })
    }

    fn decode(
        &self,
        request: &Request,
        response: Self::Response,
    ) -> Result<Response, AdapterError> {
        let response = match response {
            UserResponse::Decisions(items) => Response::Decisions(items),
            UserResponse::Wishes(items) => Response::Wishes(items),
            UserResponse::Commits(items) => Response::Commits(items),
        };
        request
            .accepts(&response)
            .then_some(response)
            .ok_or_else(|| AdapterError::InvalidResponse("response does not match request".into()))
    }
}
