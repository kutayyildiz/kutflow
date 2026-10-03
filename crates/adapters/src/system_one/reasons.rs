use std::collections::BTreeMap;

use kutflow_runtime::{Request, RequestKind};

use crate::AdapterError;

pub(super) fn criteria(request: &Request) -> Result<BTreeMap<String, String>, AdapterError> {
    let RequestKind::Reasons { reasons, .. } = &request.kind else {
        return Err(AdapterError::Unsupported);
    };
    if reasons.is_empty() || reasons.iter().any(|r| r.text.trim().is_empty()) {
        return Err(AdapterError::InvalidResponse(
            "request needs described reasons".into(),
        ));
    }
    let criteria = reasons
        .iter()
        .map(|reason| {
            (
                reason.reason_id.to_bits().get().to_string(),
                reason.text.clone(),
            )
        })
        .collect::<BTreeMap<_, _>>();
    if criteria.len() != reasons.len() {
        return Err(AdapterError::InvalidResponse(
            "request contains duplicate reason entities".into(),
        ));
    }
    if criteria.len() > 255 {
        return Err(AdapterError::InvalidResponse(
            "request exceeds 255 reasons".into(),
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
            "invalid selected reason or confidence".into(),
        ));
    }
    if probabilities.len() != criteria.len()
        || criteria.keys().any(|key| !probabilities.contains_key(key))
        || probabilities.values().any(|value| !unit(*value))
        || (probabilities.values().sum::<f64>() - 1.0).abs() > 1e-4
    {
        return Err(AdapterError::InvalidResponse(
            "probabilities must cover all reasons and sum to one".into(),
        ));
    }
    Ok(())
}

fn unit(value: f64) -> bool {
    value.is_finite() && (0.0..=1.0).contains(&value)
}
