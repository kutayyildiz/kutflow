use crate::OutputKind;
use kutflow_core::Transition;

pub(super) fn for_transition(transition: Transition) -> Option<(String, OutputKind)> {
    let (prompt, output) = match transition {
        Transition::Review
        | Transition::DiscardWish
        | Transition::DiscardCommit
        | Transition::CreateBranch
        | Transition::DiscardBranch => return None,
        Transition::ModifyWish => ("Return the modified wish.", OutputKind::Text),
        Transition::ModifyCommit => ("Return the modified commit.", OutputKind::Text),
        Transition::DissectWish => ("Return the resulting wishes.", OutputKind::List),
        Transition::AddCommit => ("Return the commits to add.", OutputKind::List),
        Transition::DissectCommit => ("Return the resulting commits.", OutputKind::List),
    };
    Some((prompt.into(), output))
}
