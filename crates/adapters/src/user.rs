use kutflow_runtime::{Request, Response};

use crate::{Adapter, AdapterError};

pub struct UserAdapter;

impl Adapter for UserAdapter {
    type Request = Request;
    type Response = Response;

    fn encode(&self, request: &Request) -> Result<Request, AdapterError> {
        Ok(request.clone())
    }

    fn decode(&self, request: &Request, response: Response) -> Result<Response, AdapterError> {
        request
            .accepts(&response)
            .then_some(response)
            .ok_or_else(|| AdapterError::InvalidResponse("response does not match request".into()))
    }
}
