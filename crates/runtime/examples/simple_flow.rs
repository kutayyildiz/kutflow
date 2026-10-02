use std::{error::Error, io};

use kutflow_runtime::{
    Decision, Request, RequestKind, Response, Runtime, Transition, config::Config,
};

fn main() -> Result<(), Box<dyn Error>> {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/basic/.kutflow/default.toml"
    );
    let config = Config::load(path)?;
    let (mut runtime, _host) = config.into_runtime()?;

    runtime.add_wish("Export notes");
    println!("Added wish: Export notes");
    runtime.sync()?;

    let review_wish = one_request(&runtime)?;
    show_request(&review_wish);
    println!("Submitting response: Review wish");
    runtime.submit(review_wish.id, decision(&review_wish, Transition::Review)?)?;
    runtime.sync()?;

    let add_commit = one_request(&runtime)?;
    show_request(&add_commit);
    println!("Submitting response: AddCommit");
    runtime.submit(add_commit.id, decision(&add_commit, Transition::AddCommit)?)?;
    runtime.sync()?;

    let generate = one_request(&runtime)?;
    show_request(&generate);
    if !matches!(&generate.kind, RequestKind::AddCommits) {
        return Err(invalid("expected an AddCommits request"));
    }
    println!("Submitting generated commit proposal: feat: export notes");
    runtime.submit(
        generate.id,
        Response::Commits(vec!["feat: export notes".into()]),
    )?;
    runtime.sync()?;

    let review_commit = one_request(&runtime)?;
    show_request(&review_commit);
    println!("Submitting response: Review commit");
    runtime.submit(
        review_commit.id,
        decision(&review_commit, Transition::Review)?,
    )?;
    runtime.sync()?;
    println!(
        "Commit proposal reviewed. Pending requests: {}",
        runtime.requests().len()
    );
    Ok(())
}

fn one_request(runtime: &Runtime) -> Result<Request, Box<dyn Error>> {
    let mut requests = runtime.requests().into_iter();
    let request = requests
        .next()
        .ok_or_else(|| invalid("runtime produced no request"))?;
    if requests.next().is_some() {
        return Err(invalid("example expected exactly one request"));
    }
    Ok(request)
}

fn decision(request: &Request, transition: Transition) -> Result<Response, Box<dyn Error>> {
    let RequestKind::Decision { targets, choices } = &request.kind else {
        return Err(invalid("expected a decision request"));
    };
    if targets.len() != 1 {
        return Err(invalid("example expects one decision target"));
    }
    let reason = choices
        .iter()
        .find(|choice| choice.transition == transition)
        .ok_or_else(|| invalid("requested transition is not available"))?
        .reason
        .clone();
    Ok(Response::Decisions(vec![Decision {
        transition,
        targets: targets.clone(),
        reason,
    }]))
}

fn show_request(request: &Request) {
    println!("\nRequest {}", request.id.0);
    println!("{}", request.prompt);
    println!("Context:\n{}", request.context);
    match &request.kind {
        RequestKind::Decision { targets, choices } => {
            println!("Targets: {}", targets.len());
            for choice in choices {
                println!("- {:?}: {}", choice.transition, choice.reason);
            }
        }
        RequestKind::AddWishes => println!("Expected response: wishes to add"),
        RequestKind::AddCommits => println!("Expected response: commits to add"),
    }
}

fn invalid(message: &'static str) -> Box<dyn Error> {
    Box::new(io::Error::new(io::ErrorKind::InvalidData, message))
}
