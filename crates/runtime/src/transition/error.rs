use std::{error::Error, fmt};

#[derive(Debug)]
pub enum TransitionError {
    ExpectedOneTarget,
    ExpectedAtLeastTwoTargets,
    ExpectedText,
    ExpectedList,
    ExpectedChoice,
    InvalidChoice,
    MissingComponent(&'static str),
    AlreadyBelongsToBranch,
    InvalidEntity,
}

impl fmt::Display for TransitionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ExpectedOneTarget => f.write_str("transition expected exactly one target"),
            Self::ExpectedAtLeastTwoTargets => {
                f.write_str("transition expected at least two targets")
            }
            Self::ExpectedText => f.write_str("transition expected a text response"),
            Self::ExpectedList => f.write_str("transition expected a list response"),
            Self::ExpectedChoice => f.write_str("transition expected a choice response"),
            Self::InvalidChoice => f.write_str("choice did not identify one transition"),
            Self::MissingComponent(component) => write!(f, "target is missing {component}"),
            Self::AlreadyBelongsToBranch => f.write_str("commit already belongs to a branch"),
            Self::InvalidEntity => f.write_str("target entity is invalid"),
        }
    }
}

impl Error for TransitionError {}
