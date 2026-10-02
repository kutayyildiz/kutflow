#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Transition {
    Review,

    ModifyWish,
    DissectWish,
    DiscardWish,

    AddCommit,
    ModifyCommit,
    DissectCommit,
    DiscardCommit,

    CreateBranch,
    DiscardBranch,
}
