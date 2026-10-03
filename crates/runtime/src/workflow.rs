use kutflow_core::Transition;

pub(super) enum OptionAction {
    Transition(Transition),
    Delegate {
        actor_question: String,
    },
    Next {
        transition: Option<Transition>,
        stage: String,
        actor_question: Option<String>,
    },
}

pub(super) struct ChoiceAction(pub OptionAction);
