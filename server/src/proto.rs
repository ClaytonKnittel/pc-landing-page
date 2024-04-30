use serde::{
  de::{self, Visitor},
  Deserialize, Serialize,
};
use tokio_util::bytes::BytesMut;

include!(concat!(env!("OUT_DIR"), "/mc_server.proto.rs"));

#[derive(Debug)]
struct BytesMutVisitor;

impl<'de> Visitor<'de> for BytesMutVisitor {
  type Value = BytesMut;

  fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
    formatter.write_str("Expecting bytes")
  }

  fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
  where
    E: serde::de::Error,
  {
    Ok(BytesMut::from(v))
  }
}

impl Serialize for ServerState {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    self.as_str_name().serialize(serializer)
  }
}

impl<'de> Deserialize<'de> for ServerState {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: serde::Deserializer<'de>,
  {
    let repr = String::deserialize(deserializer)?;
    match Self::from_str_name(&repr) {
      Some(s) => Ok(s),
      None => Err(de::Error::custom(format!(
        "Unrecognized enum variant: {repr}"
      ))),
    }
  }
}
