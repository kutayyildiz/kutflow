use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Transition {
    Review,

    AddWish,
    DiscardWish,

    AddCommit,
    DiscardCommit,

    CreateBranch,
    DiscardBranch,
}
