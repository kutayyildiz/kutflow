use kutflow_core::components::TransitionDecision;

use crate::{
    protocol::{Reason, RequestKind, State},
    request::{PendingAction, ReasonAction},
    runtime::{Runtime, RuntimeError},
};

pub(super) fn prepare_rules(runtime: &mut Runtime) -> Result<(), RuntimeError> {
    let mut rules = runtime.flow.rule.iter().collect::<Vec<_>>();
    rules.sort_by_key(|rule| rule.order);
    let mut reserved = runtime
        .pending
        .iter()
        .filter_map(|pending| pending.target())
        .collect::<Vec<_>>();
    let mut requests = Vec::new();
    for rule in rules {
        let mut selected = rule.target.query.select(runtime.world(), None)?;
        selected.sort_by_key(|entity| entity.to_bits());
        selected.retain(|entity| !reserved.contains(entity));
        for target in selected {
            let subject = Some(target);
            let state = State {
                target: rule.target.view.read_one(runtime.world(), target)?,
                context: rule
                    .context
                    .iter()
                    .map(|source| {
                        let entities = source.query.select(runtime.world(), subject)?;
                        Ok(source.view.read(runtime.world(), &entities)?)
                    })
                    .collect::<Result<Vec<_>, RuntimeError>>()?,
            };
            let projected = rule
                .choices
                .query
                .select(runtime.world(), subject)?
                .into_iter()
                .map(|entity| {
                    let transition = runtime
                        .world()
                        .get::<&TransitionDecision>(entity)
                        .map_err(|_| RuntimeError::NoReasons)?
                        .0;
                    let text = rule.choices.view.read_one(runtime.world(), entity)?;
                    Ok((
                        Reason {
                            reason_id: entity,
                            text: text.clone(),
                        },
                        ReasonAction {
                            reason_id: entity,
                            transition,
                            reason: text,
                        },
                    ))
                })
                .collect::<Result<Vec<_>, RuntimeError>>()?;
            let (reasons, actions): (Vec<_>, Vec<_>) = projected.into_iter().unzip();
            if reasons.is_empty() || reasons.iter().any(|reason| reason.text.trim().is_empty()) {
                return Err(RuntimeError::NoReasons);
            }
            requests.push((
                state,
                rule.prompt.clone(),
                RequestKind::Reasons { target, reasons },
                PendingAction::Answers(actions),
            ));
            reserved.push(target);
        }
    }
    for (state, question, kind, action) in requests {
        runtime.request(state, question, kind, action);
    }
    Ok(())
}
