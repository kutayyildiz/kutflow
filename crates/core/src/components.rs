use std::fmt;

use hecs::Entity;

use crate::Transition;

pub struct Data;

pub struct TransitionReason(pub String);
pub struct TransitionDecision(pub Transition);
pub struct Transitioned;

pub struct Name(pub String);
pub struct Description(pub String);

pub struct Wish(pub String);

pub struct Commit(pub String);
pub struct CommitHash(pub String);
pub struct BelongsToBranch(pub Entity);

pub struct Branch;

macro_rules! display_inner {
    ($($ty:ty),* $(,)?) => {
        $(
            impl fmt::Display for $ty {
                fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    self.0.fmt(f)
                }
            }
        )*
    };
}

display_inner!(
    Name,
    Description,
    TransitionReason,
    Wish,
    Commit,
    CommitHash,
);

impl fmt::Display for BelongsToBranch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.0)
    }
}
