use kutflow_core::{
    Transition,
    components::{TransitionDecision, TransitionReason, Transitioned, Wish},
    views::View,
};

use crate::{
    protocol::{RequestKind, State},
    request::{PendingAction, ResolvedAction},
    runtime::{Runtime, RuntimeError},
    transition::TransitionExt,
};

mod validation;

pub(super) fn apply(
    runtime: &mut Runtime,
    decisions: &[ResolvedAction],
) -> Result<(), RuntimeError> {
    validation::validate(runtime, decisions)?;
    let mut generated = Vec::new();
    for decision in decisions
        .iter()
        .filter(|d| matches!(d.transition, Transition::AddWish | Transition::AddCommit))
    {
        let state = generation_state(runtime, decisions, decision)?;
        generated.push((decision.transition, decision.targets.clone(), state));
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
    for (transition, targets, state) in generated {
        let kind = if transition == Transition::AddWish {
            RequestKind::Wishes
        } else {
            RequestKind::Commits
        };
        let question = if transition == Transition::AddWish {
            "Provide wishes to add."
        } else {
            "Provide commits to add."
        };
        runtime.request(
            state,
            question.into(),
            kind,
            PendingAction::Generation {
                transition,
                targets,
            },
        );
    }
    Ok(())
}

fn apply_state(runtime: &mut Runtime, decision: &ResolvedAction) -> Result<(), RuntimeError> {
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

fn generation_state(
    runtime: &Runtime,
    decisions: &[ResolvedAction],
    addition: &ResolvedAction,
) -> Result<State, RuntimeError> {
    let mut targets = Vec::new();
    let mut context = Vec::new();
    for target in &addition.targets {
        let view = if runtime.world.get::<&Wish>(*target).is_ok() {
            View::Wishes
        } else {
            View::Commits
        };
        targets.push(view.read_one(&runtime.world, *target)?);
        let mut reasons = Vec::new();
        if let Some(discard) = decisions.iter().find(|decision| {
            decision.targets.contains(target)
                && matches!(
                    decision.transition,
                    Transition::DiscardWish | Transition::DiscardCommit
                )
        }) {
            reasons.push(discard.reason.clone());
        }
        reasons.push(addition.reason.clone());
        context.push(reasons);
    }
    Ok(State { targets, context })
}

fn is_add(transition: Transition) -> bool {
    matches!(transition, Transition::AddWish | Transition::AddCommit)
}
