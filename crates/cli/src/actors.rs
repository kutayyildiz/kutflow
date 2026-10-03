use hecs::Entity;
use kutflow_core::{
    components::{AdapterRef, Description, Name},
    queries::Query,
};
use kutflow_runtime::Runtime;
use serde::Serialize;

#[derive(Clone, Serialize)]
pub struct ActorInfo {
    #[serde(with = "kutflow_core::entity")]
    pub id: Entity,
    pub name: String,
    pub description: String,
    pub adapter: String,
}

pub fn metadata(runtime: &Runtime) -> Result<Vec<ActorInfo>, String> {
    Query::Actors
        .select(runtime.world(), None)
        .map_err(|error| error.to_string())?
        .into_iter()
        .map(|entity| {
            let world = runtime.world();
            let name = world
                .get::<&Name>(entity)
                .map_err(|error| error.to_string())?;
            let description = world
                .get::<&Description>(entity)
                .map_err(|error| error.to_string())?;
            let adapter = world
                .get::<&AdapterRef>(entity)
                .map_err(|error| error.to_string())?;
            Ok(ActorInfo {
                id: entity,
                name: name.0.clone(),
                description: description.0.clone(),
                adapter: adapter.0.clone(),
            })
        })
        .collect()
}
