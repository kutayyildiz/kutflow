use hecs::Entity;
use kutflow_runtime::{RequestId, Response};
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Call {
    pub id: String,
    pub command: Command,
}

#[derive(Deserialize)]
#[serde(tag = "op", rename_all = "snake_case", deny_unknown_fields)]
pub enum Command {
    AddWish {
        wish: String,
    },
    Sync {},
    Requests {},
    Actors {},
    Submit {
        request_id: RequestId,
        response: Response,
    },
    AssignActor {
        request_id: RequestId,
        #[serde(with = "kutflow_core::entity")]
        actor_id: Entity,
    },
}
