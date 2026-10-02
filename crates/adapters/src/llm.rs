use kutflow_runtime::{Decision, Request, RequestKind, Response};
use serde_json::Value;

use crate::{Adapter, AdapterError};

mod transitions;
use transitions::{name as transition_name, parse as parse_transition};

pub struct LlmAdapter;

impl Adapter for LlmAdapter {
    type Request = String;
    type Response = String;

    fn encode(&self, request: &Request) -> Result<String, AdapterError> {
        let mut prompt = format!(
            "Context:\n{}\n\nInstruction:\n{}",
            request.context, request.prompt
        );
        match &request.kind {
            RequestKind::Decision { targets, choices } => {
                prompt.push_str("\n\nReturn JSON: {\"decisions\":[{\"transition\":\"...\",\"targets\":[target_id],\"reason\":\"...\"}]}.");
                prompt.push_str(" Allowed targets:\n");
                for target in targets {
                    prompt.push_str(&format!("- {}\n", target.to_bits().get()));
                }
                prompt.push_str("Allowed transitions and suggested reasons:\n");
                for choice in choices {
                    prompt.push_str(&format!(
                        "- {}: {}\n",
                        transition_name(choice.transition),
                        choice.reason
                    ));
                }
            }
            RequestKind::AddWishes => {
                prompt.push_str("\n\nReturn a JSON array of wish strings to add.")
            }
            RequestKind::AddCommits => {
                prompt.push_str("\n\nReturn a JSON array of commit strings.")
            }
        }
        Ok(prompt)
    }

    fn decode(&self, request: &Request, response: String) -> Result<Response, AdapterError> {
        let value: Value = serde_json::from_str(response.trim())
            .map_err(|error| AdapterError::InvalidResponse(error.to_string()))?;
        let result = match &request.kind {
            RequestKind::Decision { targets, .. } => {
                let decisions = value
                    .get("decisions")
                    .and_then(Value::as_array)
                    .ok_or_else(|| {
                        AdapterError::InvalidResponse("expected decisions array".into())
                    })?;
                let mut parsed = Vec::new();
                for decision in decisions {
                    let transition = parse_transition(
                        decision
                            .get("transition")
                            .and_then(Value::as_str)
                            .ok_or_else(|| {
                                AdapterError::InvalidResponse("missing transition".into())
                            })?,
                    )?;
                    let reason = decision
                        .get("reason")
                        .and_then(Value::as_str)
                        .ok_or_else(|| AdapterError::InvalidResponse("missing reason".into()))?
                        .to_owned();
                    let ids = decision
                        .get("targets")
                        .and_then(Value::as_array)
                        .ok_or_else(|| {
                            AdapterError::InvalidResponse("missing target ids".into())
                        })?;
                    let mapped = ids
                        .iter()
                        .map(|id| {
                            let bits = id.as_u64().ok_or_else(|| {
                                AdapterError::InvalidResponse(
                                    "target id must be an unsigned integer".into(),
                                )
                            })?;
                            targets
                                .iter()
                                .copied()
                                .find(|target| target.to_bits().get() == bits)
                                .ok_or_else(|| {
                                    AdapterError::InvalidResponse("unknown target id".into())
                                })
                        })
                        .collect::<Result<Vec<_>, _>>()?;
                    parsed.push(Decision {
                        transition,
                        targets: mapped,
                        reason,
                    });
                }
                Response::Decisions(parsed)
            }
            RequestKind::AddWishes => Response::Wishes(parse_strings(&value, "wishes")?),
            RequestKind::AddCommits => Response::Commits(parse_strings(&value, "commits")?),
        };
        request
            .accepts(&result)
            .then_some(result)
            .ok_or_else(|| AdapterError::InvalidResponse("response does not match request".into()))
    }
}

fn parse_strings(value: &Value, key: &str) -> Result<Vec<String>, AdapterError> {
    let array = value
        .as_array()
        .or_else(|| value.get(key).and_then(Value::as_array))
        .ok_or_else(|| {
            AdapterError::InvalidResponse(format!("expected JSON string array or {key} field"))
        })?;
    array
        .iter()
        .map(|item| {
            item.as_str()
                .map(str::to_owned)
                .ok_or_else(|| AdapterError::InvalidResponse("all items must be strings".into()))
        })
        .collect()
}
