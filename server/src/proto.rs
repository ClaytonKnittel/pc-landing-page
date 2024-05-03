use serde::{de, Deserialize, Serialize};

include!(concat!(env!("OUT_DIR"), "/mc_server.proto.rs"));

impl Serialize for ServerState {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    (*self as i32).serialize(serializer)
  }
}

impl<'de> Deserialize<'de> for ServerState {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: serde::Deserializer<'de>,
  {
    let repr = i32::deserialize(deserializer)?;
    Self::try_from(repr).map_err(de::Error::custom)
  }
}
