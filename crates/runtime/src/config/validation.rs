use kutflow_core::{Transition, queries::Query, views::View};

use super::loader::ConfigError;
use super::{ChoiceKind, Config, Rule, Source};

pub(super) fn validate(config: &Config) -> Result<(), ConfigError> {
    if config.flow.rule.is_empty() || config.choice.is_empty() {
        return invalid("configuration needs at least one rule and one choice");
    }
    validate_host(config)?;
    for choice in &config.choice {
        if choice.because.trim().is_empty() || !legal_choice(choice.kind, choice.transition) {
            return invalid("choice has a blank because clause or an action invalid for its kind");
        }
    }
    for rule in &config.flow.rule {
        validate_rule(config, rule)?;
    }
    Ok(())
}

fn validate_host(config: &Config) -> Result<(), ConfigError> {
    for (name, agent) in &config.host.agents {
        if name.trim().is_empty()
            || agent.adapter.trim().is_empty()
            || agent
                .model
                .as_ref()
                .is_some_and(|model| model.trim().is_empty())
        {
            return invalid("agent names, adapters, and configured models must not be blank");
        }
    }
    for name in [
        &config.host.routing.decision,
        &config.host.routing.add_wishes,
        &config.host.routing.add_commits,
    ] {
        if !config.host.agents.contains_key(name) {
            return invalid(format!("routing refers to unknown agent '{name}'"));
        }
    }
    Ok(())
}

fn validate_rule(config: &Config, rule: &Rule) -> Result<(), ConfigError> {
    if rule.prompt.trim().is_empty() {
        return invalid("rule prompt must not be blank");
    }
    let kind = target_kind(rule.target.query)
        .ok_or_else(|| ConfigError::Invalid("rule target must select wishes or commits".into()))?;
    source(&rule.target)?;
    for context in &rule.context {
        source(context)?;
    }
    let expected_query = match kind {
        ChoiceKind::Wish => Query::WishTransitionData,
        ChoiceKind::Commit => Query::CommitTransitionData,
    };
    if rule.choices.query != expected_query || rule.choices.view != View::TransitionReasons {
        return invalid(
            "rule choices must use the matching transition_data query and transition_reasons view",
        );
    }
    if !config.choice.iter().any(|choice| choice.kind == kind) {
        return invalid("rule has no configured choices for its target kind");
    }
    Ok(())
}

fn source(source: &Source) -> Result<(), ConfigError> {
    let expected = match source.query {
        Query::UnreviewedWishes | Query::ReviewedWishes | Query::WishesNeedingCommits => {
            View::Wishes
        }
        Query::UnreviewedCommits
        | Query::ReviewedCommits
        | Query::Commits
        | Query::BranchCandidates => View::Commits,
        Query::UnreviewedBranches | Query::ActiveBranches | Query::Branches => View::Branches,
        Query::WishTransitionData | Query::CommitTransitionData | Query::BranchTransitionData => {
            View::TransitionReasons
        }
        Query::BranchCommits => {
            return invalid("branch_commits context is unsupported without branch-target rules");
        }
    };
    if source.view != expected {
        return invalid("query and view do not match");
    }
    Ok(())
}

fn target_kind(query: Query) -> Option<ChoiceKind> {
    match query {
        Query::UnreviewedWishes | Query::ReviewedWishes | Query::WishesNeedingCommits => {
            Some(ChoiceKind::Wish)
        }
        Query::UnreviewedCommits
        | Query::ReviewedCommits
        | Query::Commits
        | Query::BranchCandidates => Some(ChoiceKind::Commit),
        _ => None,
    }
}

fn legal_choice(kind: ChoiceKind, transition: Transition) -> bool {
    match kind {
        ChoiceKind::Wish => matches!(
            transition,
            Transition::Review
                | Transition::DiscardWish
                | Transition::AddWish
                | Transition::AddCommit
        ),
        ChoiceKind::Commit => matches!(
            transition,
            Transition::Review
                | Transition::DiscardCommit
                | Transition::AddWish
                | Transition::AddCommit
        ),
    }
}

fn invalid<T>(message: impl Into<String>) -> Result<T, ConfigError> {
    Err(ConfigError::Invalid(message.into()))
}
