use crate::{
    rules,
    runtime::{Runtime, RuntimeError},
};

mod submissions;

pub(super) fn sync(runtime: &mut Runtime) -> Result<(), RuntimeError> {
    submissions::apply(runtime)?;
    rules::prepare_rules(runtime)
}
