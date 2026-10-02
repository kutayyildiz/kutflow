use hecs::Entity;
use kutflow_core::{
    Transition,
    components::{TransitionDecision, TransitionReason, Transitioned, Wish},
    views::View,
};

use crate::{
    protocol::{Decision, RequestKind},
    request::PendingAction,
    runtime::{Runtime, RuntimeError},
    transition::TransitionExt,
};

mod validation;

pub(super) fn apply(runtime: &mut Runtime, decisions: &[Decision]) -> Result<(), RuntimeError> {
    validation::validate(runtime, decisions)?;
    let mut generated = Vec::new();
    for decision in decisions
        .iter()
        .filter(|d| matches!(d.transition, Transition::AddWish | Transition::AddCommit))
    {
        let context = decision
            .targets
            .iter()
            .map(|target| generation_context(runtime, decisions, decision, *target))
            .collect::<Result<Vec<_>, _>>()?
            .join("\n\n");
        generated.push((decision.transition, decision.targets.clone(), context));
    }
    for decision in decisions {
        for target in &decision.targets {
            let will_discard = decisions.iter().any(|other| {
                other.targets.contains(target)
                    && matches!(
                        other.transition,
                        Transition::DiscardWish | Transition::DiscardCommit
                    )
            });
            if is_add(decision.transition) && will_discard {
                continue;
            }
            runtime
                .world
                .insert(
                    *target,
                    (
                        TransitionDecision(decision.transition),
                        TransitionReason(decision.reason.clone()),
                    ),
                )
                .map_err(|_| RuntimeError::InvalidResponse)?;
        }
        apply_state(runtime, decision)?;
    }
    for (transition, targets, context) in generated {
        let kind = if transition == Transition::AddWish {
            RequestKind::AddWishes
        } else {
            RequestKind::AddCommits
        };
        let prompt = if transition == Transition::AddWish {
            "Provide wishes to add."
        } else {
            "Provide commits to add."
        };
        runtime.request(
            context,
            prompt.into(),
            kind,
            PendingAction::Generation {
                transition,
                targets,
            },
        );
    }
    Ok(())
}

fn apply_state(runtime: &mut Runtime, decision: &Decision) -> Result<(), RuntimeError> {
    match decision.transition {
        Transition::Review => {
            for target in &decision.targets {
                runtime
                    .world
                    .insert_one(*target, Transitioned)
                    .map_err(|_| RuntimeError::InvalidResponse)?;
            }
        }
        Transition::AddWish | Transition::AddCommit => {}
        transition => {
            transition.apply(&mut runtime.world, &decision.targets)?;
            if transition == Transition::CreateBranch {
                for target in &decision.targets {
                    runtime
                        .world
                        .insert_one(*target, Transitioned)
                        .map_err(|_| RuntimeError::InvalidResponse)?;
                }
            }
        }
    }
    Ok(())
}

fn generation_context(
    runtime: &Runtime,
    decisions: &[Decision],
    addition: &Decision,
    target: Entity,
) -> Result<String, RuntimeError> {
    let view = if runtime.world.get::<&Wish>(target).is_ok() {
        View::Wishes
    } else {
        View::Commits
    };
    let source = view.read_one(&runtime.world, target)?;
    let discarded_reason = decisions
        .iter()
        .find(|d| {
            d.targets.contains(&target)
                && matches!(
                    d.transition,
                    Transition::DiscardWish | Transition::DiscardCommit
                )
        })
        .map(|d| d.reason.as_str())
        .unwrap_or_default();
    let discard = if discarded_reason.is_empty() {
        String::new()
    } else {
        format!("\nDiscard reason: {discarded_reason}")
    };
    Ok(format!(
        "Target {}:\n{source}{discard}\nAdd reason: {}",
        target.to_bits().get(),
        addition.reason
    ))
}

fn is_add(transition: Transition) -> bool {
    matches!(transition, Transition::AddWish | Transition::AddCommit)
}
