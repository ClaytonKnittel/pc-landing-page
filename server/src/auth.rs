use prost::Message;
use serde::{de::Visitor, Deserialize, Serialize};

use crate::proto::UserMap;

pub struct UserStore {
  usermap: UserMap,
}

impl Serialize for UserStore {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    serializer.serialize_bytes(self.usermap.encode_length_delimited_to_vec().as_slice())
  }
}

struct UserMapDecoder;

impl<'de> Visitor<'de> for UserMapDecoder {
  type Value = UserMap;

  fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(formatter, "Expecting proto-encoded UserMap bytes")
  }

  fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
  where
    E: serde::de::Error,
  {
    UserMap::decode(v).map_err(|err| E::custom(format!("Failed to deserialize: {err}")))
  }
}

impl<'de> Deserialize<'de> for UserStore {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: serde::Deserializer<'de>,
  {
    Ok(Self {
      usermap: deserializer.deserialize_bytes(UserMapDecoder)?,
    })
  }
}
