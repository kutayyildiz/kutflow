use crate::{
    config::RuleMode,
    protocol::{DecisionChoice, RequestKind},
    request::PendingAction,
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
            let mut context = targets
                .iter()
                .map(|entity| {
                    Ok(format!(
                        "Target {}:\n{}",
                        entity.to_bits().get(),
                        rule.target.view.read_one(runtime.world(), *entity)?
                    ))
                })
                .collect::<Result<Vec<_>, RuntimeError>>()?
                .join("\n\n");
            for source in &rule.context {
                let entities = source.query.select(runtime.world(), subject)?;
                let values = source.view.read(runtime.world(), &entities)?;
                if !values.is_empty() {
                    context.push_str(&format!("\n\n{}", values.join("\n")));
                }
            }
            let choices = rule.choices.query.select(runtime.world(), subject)?;
            let options = choices
                .iter()
                .map(|entity| {
                    Ok(DecisionChoice {
                        transition: runtime
                            .world()
                            .get::<&kutflow_core::components::TransitionDecision>(*entity)
                            .map_err(|_| RuntimeError::NoChoices)?
                            .0,
                        reason: rule.choices.view.read_one(runtime.world(), *entity)?,
                    })
                })
                .collect::<Result<Vec<_>, RuntimeError>>()?;
            if options.is_empty() {
                return Err(RuntimeError::NoChoices);
            }
            requests.push((
                context,
                rule.prompt.clone(),
                RequestKind::Decision {
                    targets: targets.clone(),
                    choices: options,
                },
                PendingAction::Decisions,
            ));
            reserved.extend(targets);
        }
    }
    for (context, prompt, kind, action) in requests {
        runtime.request(context, prompt, kind, action);
    }
    Ok(())
}
