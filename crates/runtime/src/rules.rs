use hecs::Entity;
use kutflow_core::views::View;

mod questions;

use crate::{
    config::Stage,
    protocol::State,
    runtime::{Runtime, RuntimeError},
};

pub(super) fn prepare_rules(runtime: &mut Runtime) -> Result<(), RuntimeError> {
    for initial in runtime.stages.iter().filter(|stage| stage.entry) {
        let source = initial.target.as_ref().ok_or(RuntimeError::NoOptions)?;
        let targets = source.query.select(runtime.world(), None)?;
        for target in targets {
            if !runtime.stage_for.contains_key(&target) && !has_request(runtime, target) {
                runtime.stage_for.insert(target, initial.name.clone());
            }
        }
    }
    let mut scheduled = runtime
        .stage_for
        .iter()
        .map(|(target, name)| (*target, name.clone()))
        .collect::<Vec<_>>();
    scheduled.sort_by_key(|(target, name)| {
        (
            runtime
                .stages
                .iter()
                .find(|stage| stage.name == *name)
                .map(|stage| stage.order)
                .unwrap_or(u32::MAX),
            target.to_bits(),
        )
    });
    for (target, stage_name) in scheduled {
        if has_request(runtime, target) {
            continue;
        }
        let stage = runtime
            .stages
            .iter()
            .find(|stage| stage.name == stage_name)
            .cloned()
            .ok_or(RuntimeError::NoOptions)?;
        prepare_stage(runtime, target, &stage)?;
    }
    Ok(())
}

fn prepare_stage(runtime: &mut Runtime, target: Entity, stage: &Stage) -> Result<(), RuntimeError> {
    let subject = Some(target);
    let state = State {
        target: if let Some(source) = &stage.target {
            source.view.read_one(runtime.world(), target)?
        } else if runtime
            .world()
            .get::<&kutflow_core::components::Wish>(target)
            .is_ok()
        {
            View::Wishes.read_one(runtime.world(), target)?
        } else {
            View::Commits.read_one(runtime.world(), target)?
        },
        context: stage
            .context
            .iter()
            .map(|source| {
                let entities = source.query.select(runtime.world(), subject)?;
                Ok(source.view.read(runtime.world(), &entities)?)
            })
            .collect::<Result<Vec<_>, RuntimeError>>()?,
    };
    let mut context = state.context;
    if let Some(history) = runtime.history.get(&target) {
        context.push(history.clone());
    }
    let state = State {
        target: state.target,
        context,
    };
    let questions = questions::build(runtime, target, stage)?;
    let actor = runtime
        .actor_for
        .get(&target)
        .copied()
        .or_else(|| runtime.actor_ids.get(&stage.actor).copied())
        .ok_or(RuntimeError::NoOptions)?;
    runtime.push(target, actor, state, stage, questions);
    Ok(())
}

fn has_request(runtime: &Runtime, target: Entity) -> bool {
    runtime
        .pending
        .iter()
        .any(|pending| pending.request.target == target)
}
