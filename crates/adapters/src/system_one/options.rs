use std::collections::BTreeMap;

use kutflow_runtime::{Decision, Request, RequestKind, Transition};

use crate::AdapterError;

pub(super) type OptionMapping = (String, String, Vec<Decision>);

pub(super) fn build_options(request: &Request) -> Result<Vec<OptionMapping>, AdapterError> {
    let RequestKind::Decision { targets, choices } = &request.kind else {
        return Err(AdapterError::Unsupported);
    };
    if targets.is_empty()
        || choices.is_empty()
        || choices.iter().any(|choice| choice.reason.trim().is_empty())
    {
        return Err(AdapterError::InvalidResponse(
            "decision request needs targets and described choices".into(),
        ));
    }
    let mut options = choices
        .iter()
        .enumerate()
        .map(|(index, choice)| {
            let key = format!("option_{index}");
            let description = format!("{}: {}", transition_name(choice.transition), choice.reason);
            let decision = Decision {
                transition: choice.transition,
                targets: targets.clone(),
                reason: choice.reason.clone(),
            };
            (key, description, vec![decision])
        })
        .collect::<Vec<_>>();
    for (discard_index, discard) in choices
        .iter()
        .enumerate()
        .filter(|(_, c)| is_discard(c.transition))
    {
        for (add_index, add) in choices
            .iter()
            .enumerate()
            .filter(|(_, c)| is_add(c.transition))
        {
            options.push((
                format!("replace_{discard_index}_{add_index}"),
                format!(
                    "Discard: {}; then {}: {}",
                    discard.reason,
                    transition_name(add.transition),
                    add.reason
                ),
                vec![
                    Decision {
                        transition: discard.transition,
                        targets: targets.clone(),
                        reason: discard.reason.clone(),
                    },
                    Decision {
                        transition: add.transition,
                        targets: targets.clone(),
                        reason: add.reason.clone(),
                    },
                ],
            ));
        }
    }
    if options.len() > 255 {
        return Err(AdapterError::InvalidResponse(
            "decision request exceeds 255 choices".into(),
        ));
    }
    Ok(options)
}

pub(super) fn validate_answer(
    options: &[OptionMapping],
    choice: &str,
    confidence: f64,
    probabilities: &BTreeMap<String, f64>,
) -> Result<(), AdapterError> {
    if !options.iter().any(|(key, _, _)| key == choice) || !unit(confidence) {
        return Err(AdapterError::InvalidResponse(
            "invalid selected choice or confidence".into(),
        ));
    }
    let expected = options.iter().map(|(key, _, _)| key).collect::<Vec<_>>();
    if probabilities.len() != expected.len()
        || expected.iter().any(|key| !probabilities.contains_key(*key))
        || probabilities.values().any(|value| !unit(*value))
        || (probabilities.values().sum::<f64>() - 1.0).abs() > 1e-4
    {
        return Err(AdapterError::InvalidResponse(
            "probabilities must cover all choices and sum to one".into(),
        ));
    }
    Ok(())
}

fn unit(value: f64) -> bool {
    value.is_finite() && (0.0..=1.0).contains(&value)
}
fn is_discard(value: Transition) -> bool {
    matches!(value, Transition::DiscardWish | Transition::DiscardCommit)
}
fn is_add(value: Transition) -> bool {
    matches!(value, Transition::AddWish | Transition::AddCommit)
}

fn transition_name(value: Transition) -> &'static str {
    match value {
        Transition::Review => "review",
        Transition::AddWish => "add wish",
        Transition::DiscardWish => "discard wish",
        Transition::AddCommit => "add commit",
        Transition::DiscardCommit => "discard commit",
        Transition::CreateBranch => "create branch",
        Transition::DiscardBranch => "discard branch",
    }
}
