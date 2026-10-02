pub mod jev;
pub mod llm;
pub mod user;

use std::{error::Error, fmt};

use kutflow_runtime::{Request, Response};

pub trait Adapter {
    type Request;
    type Response;

    fn encode(&self, request: &Request) -> Result<Self::Request, AdapterError>;

    fn decode(&self, request: &Request, response: Self::Response)
    -> Result<Response, AdapterError>;
}

#[derive(Debug)]
pub enum AdapterError {
    Unsupported,
    InvalidResponse(String),
}

impl fmt::Display for AdapterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unsupported => write!(f, "request is not supported by this adapter"),
            Self::InvalidResponse(message) => write!(f, "invalid response: {message}"),
        }
    }
}

impl Error for AdapterError {}
