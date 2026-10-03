use crate::{protocol::Response, request::Pending};
use kutflow_core::components::Discarded;

use crate::runtime::{Runtime, RuntimeError};

mod decisions;

pub(super) fn validate_response(
    runtime: &Runtime,
    pending: &Pending,
    response: &Response,
) -> Result<(), RuntimeError> {
    decisions::validate_response(runtime, pending, response)
}

pub(super) fn apply(runtime: &mut Runtime) -> Result<(), RuntimeError> {
    let mut pending = std::mem::take(&mut runtime.pending);
    while !pending.is_empty() {
        let mut item = pending.remove(0);
        let Some(response) = item.response.clone() else {
            runtime.pending.push(item);
            continue;
        };
        match decisions::apply(runtime, &item, &response) {
            Ok(Some(actor)) => {
                item.request.actor = actor;
                if let Some(history) = runtime.history.get(&item.request.target) {
                    item.request.state.context.push(
                        history
                            .iter()
                            .rev()
                            .take(2)
                            .cloned()
                            .collect::<Vec<_>>()
                            .into_iter()
                            .rev()
                            .collect(),
                    );
                }
                item.response = None;
                item.handoffs += 1;
                runtime.pending.push(item);
            }
            Ok(None) => {
                let target = item.request.target;
                if runtime.world.get::<&Discarded>(target).is_ok()
                    && !runtime.stage_for.contains_key(&target)
                {
                    runtime.stage_for.remove(&target);
                    runtime.actor_for.remove(&target);
                    pending.retain(|other| other.request.target != target);
                    runtime
                        .pending
                        .retain(|other| other.request.target != target);
                }
            }
            Err(error) => {
                item.response = None;
                runtime.pending.push(item);
                runtime.pending.extend(pending);
                return Err(error);
            }
        }
    }
    Ok(())
}
