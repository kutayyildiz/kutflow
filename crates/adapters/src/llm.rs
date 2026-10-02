use kutflow_runtime::{OutputKind, Request, Response};

use crate::{Adapter, AdapterError};

pub struct LlmAdapter;

impl Adapter for LlmAdapter {
    type Request = String;
    type Response = String;

    fn encode(&self, request: &Request) -> Result<Self::Request, AdapterError> {
        let mut prompt = format!(
            "Context:\n{}\n\nInstruction:\n{}",
            request.context, request.prompt
        );

        match &request.output {
            OutputKind::Text => {}

            OutputKind::List => {
                prompt.push_str("\n\nReturn one item per line.");
            }

            OutputKind::Choice(choices) => {
                prompt.push_str("\n\nChoose exactly one of:\n");

                for choice in choices {
                    prompt.push_str("- ");
                    prompt.push_str(choice);
                    prompt.push('\n');
                }

                prompt.push_str("\nReturn only the selected choice.");
            }

            OutputKind::Probability => {
                prompt.push_str("\n\nReturn only a probability between 0.0 and 1.0.");
            }

            OutputKind::Score(scale) => {
                prompt.push_str("\n\nOrdered scale:\n");

                for item in scale {
                    prompt.push_str("- ");
                    prompt.push_str(item);
                    prompt.push('\n');
                }

                prompt.push_str("\nReturn only a normalized score between 0.0 and 1.0.");
            }
        }

        Ok(prompt)
    }

    fn decode(
        &self,
        request: &Request,
        response: Self::Response,
    ) -> Result<Response, AdapterError> {
        let response = response.trim();

        match &request.output {
            OutputKind::Text => Ok(Response::Text(response.to_owned())),

            OutputKind::List => Ok(Response::List(
                response
                    .lines()
                    .map(str::trim)
                    .filter(|line| !line.is_empty())
                    .map(str::to_owned)
                    .collect(),
            )),

            OutputKind::Choice(choices) => {
                if choices.iter().any(|choice| choice == response) {
                    Ok(Response::Choice(response.to_owned()))
                } else {
                    Err(AdapterError::InvalidResponse(response.to_owned()))
                }
            }

            OutputKind::Probability => {
                let value = parse_unit_f64(response)?;
                Ok(Response::Probability(value))
            }

            OutputKind::Score(_) => {
                let value = parse_unit_f64(response)?;
                Ok(Response::Score(value))
            }
        }
    }
}

fn parse_unit_f64(value: &str) -> Result<f64, AdapterError> {
    let value: f64 = value
        .parse()
        .map_err(|_| AdapterError::InvalidResponse(value.to_owned()))?;

    if !(0.0..=1.0).contains(&value) {
        return Err(AdapterError::InvalidResponse(value.to_string()));
    }

    Ok(value)
}
