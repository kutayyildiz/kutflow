use crate::{
    rules,
    runtime::{Runtime, RuntimeError},
};

mod submissions;

pub(crate) fn validate_submission(
    runtime: &Runtime,
    pending: &crate::request::Pending,
    response: &crate::protocol::Response,
) -> Result<(), RuntimeError> {
    submissions::validate_response(runtime, pending, response)
}

pub(super) fn sync(runtime: &mut Runtime) -> Result<(), RuntimeError> {
    submissions::apply(runtime)?;
    rules::prepare_rules(runtime)
}
