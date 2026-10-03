use crate::{
    actors::metadata,
    protocol::{Call, Command},
};
use kutflow_runtime::{Runtime, config::Config};
use std::{
    env,
    io::{self, BufRead, Write},
    path::PathBuf,
};

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let (mut runtime, _) = Config::load(config_path()?)?.into_runtime()?;
    let actors = metadata(&runtime).map_err(io::Error::other)?;
    write_json(
        &serde_json::json!({"id":null,"ok":true,"event":"ready","result":{"actors":actors}}),
    )?;
    for line in io::stdin().lock().lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let value = match serde_json::from_str::<serde_json::Value>(&line) {
            Ok(value) => value,
            Err(error) => {
                write_json(&failure(serde_json::Value::Null, error.to_string()))?;
                continue;
            }
        };
        let correlation = value.get("id").cloned().unwrap_or(serde_json::Value::Null);
        match serde_json::from_value::<Call>(value) {
            Ok(call) => match execute(&mut runtime, call.command) {
                Ok(data) => write_json(&serde_json::json!({"id":call.id,"ok":true,"result":data}))?,
                Err(error) => write_json(&failure(call.id.into(), error))?,
            },
            Err(error) => write_json(&failure(correlation, error.to_string()))?,
        }
    }
    Ok(())
}

fn execute(runtime: &mut Runtime, command: Command) -> Result<serde_json::Value, String> {
    let result = match command {
        Command::AddWish { wish } => {
            if wish.trim().is_empty() {
                return Err("wish must not be blank".into());
            }
            let entity = runtime.add_wish(wish);
            serde_json::json!({"entity":kutflow_core::entity::to_string(&entity)})
        }
        Command::Sync {} => {
            runtime.sync().map_err(|error| error.to_string())?;
            requests(runtime)
        }
        Command::Requests {} => requests(runtime),
        Command::Actors {} => serde_json::json!({"actors":metadata(runtime)?}),
        Command::Submit {
            request_id,
            response,
        } => {
            runtime
                .submit(request_id, response)
                .map_err(|error| error.to_string())?;
            serde_json::json!({"submitted":request_id})
        }
        Command::AssignActor {
            request_id,
            actor_id,
        } => {
            runtime
                .assign_actor(request_id, actor_id)
                .map_err(|error| error.to_string())?;
            serde_json::json!({"assigned":kutflow_core::entity::to_string(&actor_id)})
        }
    };
    Ok(result)
}

fn requests(runtime: &Runtime) -> serde_json::Value {
    serde_json::json!({"requests":runtime.requests()})
}
fn failure(id: serde_json::Value, error: String) -> serde_json::Value {
    serde_json::json!({"id":id,"ok":false,"error":error})
}
fn write_json(value: &serde_json::Value) -> io::Result<()> {
    let stdout = io::stdout();
    let mut output = stdout.lock();
    serde_json::to_writer(&mut output, value)?;
    output.write_all(b"\n")?;
    output.flush()
}
fn config_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    match args.as_slice() {
        [flag, path] if flag == "--config" => Ok(path.into()),
        [] => Ok(env::var("KUTFLOW_CONFIG")
            .unwrap_or_else(|_| "extensions/pi/default.toml".into())
            .into()),
        _ => Err("usage: kutflow-cli [--config <path>]".into()),
    }
}
