use std::{error::Error, io};

use kutflow_runtime::{Answer, Request, RequestKind, Response, Runtime, config::Config};

fn main() -> Result<(), Box<dyn Error>> {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/basic/.kutflow/default.toml"
    );
    let config = Config::load(path)?;
    let (mut runtime, _host) = config.into_runtime()?;

    runtime.add_wish("Export notes");
    runtime.sync()?;

    let review_wish = one_request(&runtime)?;
    let review_reason =
        "Accept this wish because it is clear, coherent, and describes one intended outcome.";
    let review_response = selection(&review_wish, review_reason)?;
    println!("{review_wish:#?}");
    println!("{review_response:#?}");
    runtime.submit(review_wish.id, review_response)?;
    runtime.sync()?;

    let add_commit = one_request(&runtime)?;
    let add_commit_reason =
        "Add commit proposals because the accepted wish has no implementation proposal yet.";
    let add_commit_response = selection(&add_commit, add_commit_reason)?;
    println!("{add_commit:#?}");
    println!("{add_commit_response:#?}");
    runtime.submit(add_commit.id, add_commit_response)?;
    runtime.sync()?;

    let generate = one_request(&runtime)?;
    if !matches!(&generate.kind, RequestKind::Commits) {
        return Err(invalid("expected a commits request"));
    }
    let generation_response = Response::Commits(vec!["feat: export notes".into()]);
    println!("{generate:#?}");
    println!("{generation_response:#?}");
    runtime.submit(generate.id, generation_response)?;
    runtime.sync()?;

    let review_commit = one_request(&runtime)?;
    let review_commit_reason =
        "Accept this proposal because it is clear, coherent, and independently reviewable.";
    let review_commit_response = selection(&review_commit, review_commit_reason)?;
    println!("{review_commit:#?}");
    println!("{review_commit_response:#?}");
    runtime.submit(review_commit.id, review_commit_response)?;
    runtime.sync()?;
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

fn selection(request: &Request, reason: &str) -> Result<Response, Box<dyn Error>> {
    let RequestKind::Reasons { target, reasons } = &request.kind else {
        return Err(invalid("expected a reason request"));
    };
    let selected = reasons
        .iter()
        .find(|candidate| candidate.text == reason)
        .ok_or_else(|| invalid("configured reason is not available"))?;
    Ok(Response::Answers(vec![Answer {
        reason_id: selected.reason_id,
        target: *target,
    }]))
}

fn invalid(message: &'static str) -> Box<dyn Error> {
    Box::new(io::Error::new(io::ErrorKind::InvalidData, message))
}
