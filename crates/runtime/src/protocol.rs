use hecs::Entity;
use kutflow_core::Transition;

#[derive(Debug, Clone)]
pub struct Request {
    pub id: RequestId,
    pub context: String,
    pub prompt: String,
    pub kind: RequestKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RequestId(pub u64);

#[derive(Debug, Clone)]
pub enum RequestKind {
    Decision {
        targets: Vec<Entity>,
        choices: Vec<DecisionChoice>,
    },
    AddWishes,
    AddCommits,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecisionChoice {
    pub transition: Transition,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Decision {
    pub transition: Transition,
    pub targets: Vec<Entity>,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Response {
    Decisions(Vec<Decision>),
    Wishes(Vec<String>),
    Commits(Vec<String>),
}

impl Request {
    pub fn accepts(&self, response: &Response) -> bool {
        match (&self.kind, response) {
            (RequestKind::AddWishes, Response::Wishes(values))
            | (RequestKind::AddCommits, Response::Commits(values)) => {
                values.iter().all(|value| !value.trim().is_empty())
            }
            (RequestKind::Decision { targets, choices }, Response::Decisions(decisions)) => {
                valid_decisions(targets, choices, decisions)
            }
            _ => false,
        }
    }
}

fn valid_decisions(targets: &[Entity], choices: &[DecisionChoice], decisions: &[Decision]) -> bool {
    if decisions.is_empty() {
        return false;
    }
    for (index, decision) in decisions.iter().enumerate() {
        if decision.reason.trim().is_empty()
            || !choices
                .iter()
                .any(|choice| choice.transition == decision.transition)
            || decision.targets.is_empty()
        {
            return false;
        }
        if decision
            .targets
            .iter()
            .enumerate()
            .any(|(i, target)| decision.targets[..i].contains(target))
        {
            return false;
        }
        for target in &decision.targets {
            if !targets.contains(target)
                || decisions[..index].iter().any(|prior| {
                    prior.transition == decision.transition && prior.targets.contains(target)
                })
            {
                return false;
            }
        }
    }
    for target in targets {
        let selected = decisions
            .iter()
            .enumerate()
            .filter_map(|(index, item)| {
                item.targets
                    .contains(target)
                    .then_some((index, item.transition))
            })
            .collect::<Vec<_>>();
        match selected.as_slice() {
            [] | [(_, _)] => {}
            [(first_index, first), (second_index, second)] => {
                let matching_add = matches!(
                    (*first, *second),
                    (
                        Transition::DiscardWish | Transition::DiscardCommit,
                        Transition::AddWish | Transition::AddCommit
                    )
                );
                if !matching_add || first_index >= second_index {
                    return false;
                }
            }
            _ => return false,
        }
    }
    true
}
