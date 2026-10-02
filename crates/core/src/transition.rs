#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Transition {
    Review,

    AddWish,
    DiscardWish,

    AddCommit,
    DiscardCommit,

    CreateBranch,
    DiscardBranch,
}
