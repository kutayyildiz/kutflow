use kutflow_core::components::TransitionDecision;

use crate::{
    config::RuleMode,
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
        .flat_map(|pending| pending.targets().to_vec())
        .collect::<Vec<_>>();
    let mut requests = Vec::new();
    for rule in rules {
        let mut selected = rule.target.query.select(runtime.world(), None)?;
        selected.sort_by_key(|entity| entity.to_bits());
        selected.retain(|entity| !reserved.contains(entity));
        let groups = match rule.mode {
            RuleMode::PerEntity => selected.iter().map(|entity| vec![*entity]).collect(),
            RuleMode::Batch if selected.is_empty() => Vec::new(),
            RuleMode::Batch => vec![selected],
        };
        for targets in groups {
            let subject = if matches!(rule.mode, RuleMode::PerEntity) {
                targets.first().copied()
            } else {
                None
            };
            let target_values = rule.target.view.read(runtime.world(), &targets)?;
            let context = rule
                .context
                .iter()
                .map(|source| {
                    let entities = source.query.select(runtime.world(), subject)?;
                    Ok(source.view.read(runtime.world(), &entities)?)
                })
                .collect::<Result<Vec<_>, RuntimeError>>()?;
            let state = State {
                targets: target_values,
                context,
            };
            let reason_entities = rule.choices.query.select(runtime.world(), subject)?;
            let projected = reason_entities
                .iter()
                .map(|entity| {
                    let transition = runtime
                        .world()
                        .get::<&TransitionDecision>(*entity)
                        .map_err(|_| RuntimeError::NoReasons)?
                        .0;
                    let reason = rule.choices.view.read_one(runtime.world(), *entity)?;
                    Ok((
                        Reason {
                            entity: *entity,
                            text: reason.clone(),
                        },
                        ReasonAction {
                            entity: *entity,
                            transition,
                            reason,
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
                RequestKind::Reasons {
                    targets: targets.clone(),
                    reasons,
                },
                PendingAction::Answers(actions),
            ));
            reserved.extend(targets);
        }
    }
    for (state, question, kind, action) in requests {
        runtime.request(state, question, kind, action);
    }
    Ok(())
}
