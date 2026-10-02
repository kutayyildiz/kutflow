use std::{error::Error, fmt};

#[derive(Debug)]
pub enum TransitionError {
    ExpectedOneTarget,
    ExpectedAtLeastTwoTargets,
    RequiresRequest,
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
            Self::RequiresRequest => f.write_str("transition requires a runtime request"),
            Self::MissingComponent(component) => write!(f, "target is missing {component}"),
            Self::AlreadyBelongsToBranch => f.write_str("commit already belongs to a branch"),
            Self::InvalidEntity => f.write_str("target entity is invalid"),
        }
    }
}

impl Error for TransitionError {}
