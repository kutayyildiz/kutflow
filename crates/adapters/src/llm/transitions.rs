use kutflow_runtime::Transition;

use crate::AdapterError;

pub(super) fn name(transition: Transition) -> &'static str {
    match transition {
        Transition::Review => "review",
        Transition::AddWish => "add_wish",
        Transition::DiscardWish => "discard_wish",
        Transition::AddCommit => "add_commit",
        Transition::DiscardCommit => "discard_commit",
        Transition::CreateBranch => "create_branch",
        Transition::DiscardBranch => "discard_branch",
    }
}

pub(super) fn parse(value: &str) -> Result<Transition, AdapterError> {
    match value {
        "review" => Ok(Transition::Review),
        "add_wish" => Ok(Transition::AddWish),
        "discard_wish" => Ok(Transition::DiscardWish),
        "add_commit" => Ok(Transition::AddCommit),
        "discard_commit" => Ok(Transition::DiscardCommit),
        "create_branch" => Ok(Transition::CreateBranch),
        "discard_branch" => Ok(Transition::DiscardBranch),
        _ => Err(AdapterError::InvalidResponse(format!(
            "unknown transition {value}"
        ))),
    }
}
