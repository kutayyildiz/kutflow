use std::{collections::BTreeMap, error::Error, io};

use kutflow_runtime::{Answer, QuestionKind, Request, Response, Runtime, config::Config};

fn main() -> Result<(), Box<dyn Error>> {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/basic/.kutflow/default.toml"
    );
    let (mut runtime, host) = Config::load(path)?.into_runtime()?;
    let _host_settings = host;
    runtime.add_wish("Export notes");
    runtime.sync()?;

    let review = one_request(&runtime)?;
    let answer = response(
        &review,
        "Accept this wish because",
        Some("generator:"),
        Some("user:"),
    )?;
    println!("{review:#?}\n{answer:#?}");
    runtime.submit(review.id, answer)?;
    runtime.sync()?;

    let plan = one_request(&runtime)?;
    let answer = response(
        &plan,
        "Add commit proposals because",
        Some("generator:"),
        Some("user:"),
    )?;
    println!("{plan:#?}\n{answer:#?}");
    runtime.submit(plan.id, answer)?;
    runtime.sync()?;

    let generate = one_request(&runtime)?;
    let generated = Response {
        answers: BTreeMap::from([(
            "commits".into(),
            Answer::Texts(vec!["feat: export notes".into()]),
        )]),
    };
    println!("{generate:#?}\n{generated:#?}");
    runtime.submit(generate.id, generated)?;
    runtime.sync()?;

    let review_commit = one_request(&runtime)?;
    let answer = response(
        &review_commit,
        "Accept this proposal because",
        Some("generator:"),
        Some("user:"),
    )?;
    println!("{review_commit:#?}\n{answer:#?}");
    runtime.submit(review_commit.id, answer)?;
    runtime.sync()?;
    Ok(())
}

fn one_request(runtime: &Runtime) -> Result<Request, Box<dyn Error>> {
    let mut requests = runtime.requests().into_iter();
    let request = requests
        .next()
        .ok_or_else(|| invalid("runtime produced no request"))?;
    if requests.next().is_some() {
        return Err(invalid("example expected one request"));
    }
    Ok(request)
}

fn response(
    request: &Request,
    reason: &str,
    generation_actor: Option<&str>,
    delegation_actor: Option<&str>,
) -> Result<Response, Box<dyn Error>> {
    let mut answers = BTreeMap::new();
    answers.insert(
        "decision".into(),
        Answer::Choice(select(request, "decision", reason)?),
    );
    if let Some(actor) = generation_actor {
        answers.insert(
            "actor_for_generation".into(),
            Answer::Choice(select(request, "actor_for_generation", actor)?),
        );
    }
    if let Some(actor) = delegation_actor {
        answers.insert(
            "actor_for_delegation".into(),
            Answer::Choice(select(request, "actor_for_delegation", actor)?),
        );
    }
    Ok(Response { answers })
}

fn select(request: &Request, question: &str, text: &str) -> Result<hecs::Entity, Box<dyn Error>> {
    let options = request
        .questions
        .get(question)
        .ok_or_else(|| invalid("question missing"))?;
    let QuestionKind::Choice(options) = &options.kind else {
        return Err(invalid("question is not a choice"));
    };
    options
        .iter()
        .find(|option| option.text.starts_with(text))
        .map(|option| option.entity)
        .ok_or_else(|| invalid("option missing"))
}

fn invalid(message: &'static str) -> Box<dyn Error> {
    Box::new(io::Error::new(io::ErrorKind::InvalidData, message))
}
