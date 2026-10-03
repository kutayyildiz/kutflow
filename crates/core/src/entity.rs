use hecs::Entity;
use serde::{Deserialize, Deserializer, Serializer, de::Error};

pub fn serialize<S: Serializer>(entity: &Entity, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&to_string(entity))
}

pub fn to_string(entity: &Entity) -> String {
    entity.to_bits().get().to_string()
}

pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Entity, D::Error> {
    let value = String::deserialize(deserializer)?;
    let bits = value.parse::<u64>().map_err(D::Error::custom)?;
    Entity::from_bits(bits).ok_or_else(|| D::Error::custom("invalid entity ID"))
}
