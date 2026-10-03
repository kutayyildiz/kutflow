use kutflow_runtime::{Answer, Request, RequestKind, Response};
use serde::Deserialize;
use serde_json::Value;

use crate::{Adapter, AdapterError};

pub struct LlmAdapter;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AnswerResponse {
    answers: Vec<Selection>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Selection {
    reason_id: u64,
    target: u64,
}

impl Adapter for LlmAdapter {
    type Request = String;
    type Response = String;

    fn encode(&self, request: &Request) -> Result<String, AdapterError> {
        let state = serde_json::to_string_pretty(&request.state)
            .map_err(|error| AdapterError::InvalidResponse(error.to_string()))?;
        let mut prompt = format!("State:\n{}\n\nQuestion:\n{}", state, request.question);
        match &request.kind {
            RequestKind::Reasons { target, reasons } => {
                prompt.push_str("\n\nWhich reason best fits this state? Return JSON: {\"answers\":[{\"reason_id\":reason_id,\"target\":target_id}]}.");
                prompt.push_str(&format!(" Target ID: {}\n", target.to_bits().get()));
                prompt.push_str("Reason IDs and descriptions:\n");
                for reason in reasons {
                    prompt.push_str(&format!(
                        "- {}: {}\n",
                        reason.reason_id.to_bits().get(),
                        reason.text
                    ));
                }
            }
            RequestKind::Wishes => {
                prompt.push_str("\n\nReturn a JSON array of wish strings to add.")
            }
            RequestKind::Commits => prompt.push_str("\n\nReturn a JSON array of commit strings."),
        }
        Ok(prompt)
    }

    fn decode(&self, request: &Request, response: String) -> Result<Response, AdapterError> {
        let value: Value = serde_json::from_str(response.trim())
            .map_err(|error| AdapterError::InvalidResponse(error.to_string()))?;
        let result = match &request.kind {
            RequestKind::Reasons { target, reasons } => {
                let parsed: AnswerResponse = serde_json::from_value(value)
                    .map_err(|error| AdapterError::InvalidResponse(error.to_string()))?;
                let answers = parsed
                    .answers
                    .into_iter()
                    .map(|selection| {
                        let reason = reasons
                            .iter()
                            .find(|reason| reason.reason_id.to_bits().get() == selection.reason_id)
                            .map(|reason| reason.reason_id)
                            .ok_or_else(|| {
                                AdapterError::InvalidResponse("unknown reason entity".into())
                            })?;
                        if target.to_bits().get() != selection.target {
                            return Err(AdapterError::InvalidResponse("unknown target id".into()));
                        }
                        Ok(Answer {
                            reason_id: reason,
                            target: *target,
                        })
                    })
                    .collect::<Result<Vec<_>, AdapterError>>()?;
                Response::Answers(answers)
            }
            RequestKind::Wishes => Response::Wishes(parse_strings(&value, "wishes")?),
            RequestKind::Commits => Response::Commits(parse_strings(&value, "commits")?),
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
