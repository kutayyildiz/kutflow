use std::collections::BTreeMap;

use kutflow_runtime::{Question, QuestionKind};

use crate::AdapterError;

pub(super) fn criteria(question: &Question) -> Result<BTreeMap<String, String>, AdapterError> {
    let QuestionKind::Choice(options) = &question.kind else {
        return Err(AdapterError::Unsupported);
    };
    if options.is_empty() || options.iter().any(|option| option.text.trim().is_empty()) {
        return Err(AdapterError::InvalidResponse(
            "question needs described options".into(),
        ));
    }
    let criteria = options
        .iter()
        .map(|option| {
            (
                option.entity.to_bits().get().to_string(),
                option.text.clone(),
            )
        })
        .collect::<BTreeMap<_, _>>();
    if criteria.len() != options.len() {
        return Err(AdapterError::InvalidResponse(
            "question contains duplicate options".into(),
        ));
    }
    if criteria.len() > 255 {
        return Err(AdapterError::InvalidResponse(
            "question exceeds 255 options".into(),
        ));
    }
    Ok(criteria)
}

pub(super) fn validate_answer(
    criteria: &BTreeMap<String, String>,
    selected: &str,
    confidence: f64,
    probabilities: &BTreeMap<String, f64>,
) -> Result<(), AdapterError> {
    if !criteria.contains_key(selected) || !unit(confidence) {
        return Err(AdapterError::InvalidResponse(
            "invalid selected option or confidence".into(),
        ));
    }
    if probabilities.len() != criteria.len()
        || criteria.keys().any(|key| !probabilities.contains_key(key))
        || probabilities.values().any(|value| !unit(*value))
        || (probabilities.values().sum::<f64>() - 1.0).abs() > 1e-4
    {
        return Err(AdapterError::InvalidResponse(
            "probabilities must cover all options and sum to one".into(),
        ));
    }
    Ok(())
}

fn unit(value: f64) -> bool {
    value.is_finite() && (0.0..=1.0).contains(&value)
}
