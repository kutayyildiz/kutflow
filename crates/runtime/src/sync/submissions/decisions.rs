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
    let generated = decisions
        .iter()
        .filter(|decision| is_add(decision.transition))
        .map(|addition| {
            Ok((
                addition.transition,
                addition.target,
                generation_state(runtime, decisions, addition)?,
            ))
        })
        .collect::<Result<Vec<_>, RuntimeError>>()?;

    for decision in decisions {
        if is_add(decision.transition)
            && decisions.iter().any(|other| {
                other.target == decision.target
                    && matches!(
                        other.transition,
                        Transition::DiscardWish | Transition::DiscardCommit
                    )
            })
        {
            continue;
        }
        runtime
            .world
            .insert(
                decision.target,
                (
                    TransitionDecision(decision.transition),
                    TransitionReason(decision.reason.clone()),
                ),
            )
            .map_err(|_| RuntimeError::InvalidResponse)?;
        apply_state(runtime, decision)?;
    }

    for (transition, target, state) in generated {
        let (kind, question) = if transition == Transition::AddWish {
            (RequestKind::Wishes, "Provide wishes to add.")
        } else {
            (RequestKind::Commits, "Provide commits to add.")
        };
        runtime.request(
            state,
            question.into(),
            kind,
            PendingAction::Generation { transition, target },
        );
    }
    Ok(())
}

fn apply_state(runtime: &mut Runtime, decision: &ResolvedAction) -> Result<(), RuntimeError> {
    match decision.transition {
        Transition::Review => runtime
            .world
            .insert_one(decision.target, Transitioned)
            .map_err(|_| RuntimeError::InvalidResponse)?,
        Transition::AddWish | Transition::AddCommit => {}
        transition => transition.apply(&mut runtime.world, &[decision.target])?,
    }
    Ok(())
}

fn generation_state(
    runtime: &Runtime,
    decisions: &[ResolvedAction],
    addition: &ResolvedAction,
) -> Result<State, RuntimeError> {
    let target = if runtime.world.get::<&Wish>(addition.target).is_ok() {
        View::Wishes.read_one(&runtime.world, addition.target)?
    } else {
        View::Commits.read_one(&runtime.world, addition.target)?
    };
    let mut context = Vec::new();
    if let Some(discard) = decisions.iter().find(|decision| {
        decision.target == addition.target
            && matches!(
                decision.transition,
                Transition::DiscardWish | Transition::DiscardCommit
            )
    }) {
        context.push(discard.reason.clone());
    }
    context.push(addition.reason.clone());
    Ok(State {
        target,
        context: vec![context],
    })
}

fn is_add(transition: Transition) -> bool {
    matches!(transition, Transition::AddWish | Transition::AddCommit)
}
