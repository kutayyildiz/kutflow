use hecs::Entity;

use crate::{
    OutputKind,
    config::RuleMode,
    request::PendingAction,
    runtime::{Runtime, RuntimeError},
};

pub(super) fn prepare_rules(runtime: &mut Runtime) -> Result<(), RuntimeError> {
    let mut rules = runtime.flow.rule.iter().collect::<Vec<_>>();
    rules.sort_by_key(|rule| rule.order);
    let mut reserved = runtime
        .pending
        .iter()
        .flat_map(|pending| match &pending.action {
            PendingAction::Rule { targets, .. } | PendingAction::Transition { targets, .. } => {
                targets.clone()
            }
        })
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
            let mut context = rule.target.view.read(runtime.world(), &targets)?.join("\n");
            for source in &rule.context {
                let entities = source.query.select(runtime.world(), subject)?;
                let values = source.view.read(runtime.world(), &entities)?;
                if !values.is_empty() {
                    context.push_str(&format!("\n\n{}", values.join("\n")));
                }
            }
            let choices = rule.choices.query.select(runtime.world(), subject)?;
            let labels = rule.choices.view.read(runtime.world(), &choices)?;
            if labels.is_empty() {
                return Err(RuntimeError::NoChoices);
            }
            if labels
                .iter()
                .any(|label| labels.iter().filter(|item| *item == label).count() > 1)
            {
                return Err(RuntimeError::AmbiguousChoice("duplicate labels".into()));
            }
            requests.push((
                context,
                rule.prompt.clone(),
                labels.clone(),
                PendingAction::Rule {
                    targets: targets.clone(),
                    choices,
                    labels,
                },
            ));
            reserved.extend(targets);
        }
    }
    for (context, prompt, labels, action) in requests {
        runtime.request(context, prompt, OutputKind::Choice(labels), action);
    }
    Ok(())
}

pub(super) fn is_pending(runtime: &Runtime, entity: Entity) -> bool {
    runtime.pending.iter().any(|pending| match &pending.action {
        PendingAction::Transition { targets, .. } | PendingAction::Rule { targets, .. } => {
            targets.contains(&entity)
        }
    })
}
