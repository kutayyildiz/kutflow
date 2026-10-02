use kutflow_runtime::{Decision, Request, RequestKind, Response};

use crate::{Adapter, AdapterError};

pub struct SystemOneAdapter;

pub enum SystemOneRequest {
    Decisions {
        state: String,
        prompt: String,
        kind: RequestKind,
    },
}

pub enum SystemOneResult {
    Decisions(Vec<Decision>),
}

impl Adapter for SystemOneAdapter {
    type Request = SystemOneRequest;
    type Response = SystemOneResult;

    fn encode(&self, request: &Request) -> Result<Self::Request, AdapterError> {
        match &request.kind {
            RequestKind::Decision { .. } => Ok(SystemOneRequest::Decisions {
                state: request.context.clone(),
                prompt: request.prompt.clone(),
                kind: request.kind.clone(),
            }),
            RequestKind::AddWishes | RequestKind::AddCommits => Err(AdapterError::Unsupported),
        }
    }

    fn decode(
        &self,
        request: &Request,
        response: Self::Response,
    ) -> Result<Response, AdapterError> {
        let SystemOneResult::Decisions(items) = response;
        let response = Response::Decisions(items);
        request
            .accepts(&response)
            .then_some(response)
            .ok_or_else(|| AdapterError::InvalidResponse("response does not match request".into()))
    }
}
