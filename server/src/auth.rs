use std::collections::hash_map;

use prost::Message;
use serde::{de::Visitor, Deserialize, Serialize};

use crate::{
  error::{McError, McResult},
  proto::{User, UserMap},
};

pub struct UserStore {
  usermap: UserMap,
}

impl UserStore {
  pub fn new() -> Self {
    Self {
      usermap: UserMap::default(),
    }
  }

  #[cfg(test)]
  pub fn num_users(&self) -> usize {
    self.usermap.users.len()
  }

  pub fn add_user(&mut self, username: String, password: String) -> McResult<()> {
    if username.is_empty() {
      return Err(McError::InvalidOp("Cannot have empty username".to_owned()));
    }

    match self.usermap.users.entry(username.clone()) {
      hash_map::Entry::Occupied(_) => Err(McError::InvalidOp(format!(
        "User {username} already exists"
      ))),
      hash_map::Entry::Vacant(entry) => {
        entry.insert(User {
          password: Some(password),
        });
        Ok(())
      }
    }
  }

  pub fn find_user(&self, username: &str) -> Option<&User> {
    self.usermap.users.get(username)
  }
}

impl Default for UserStore {
  fn default() -> Self {
    Self::new()
  }
}

impl Serialize for UserStore {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    serializer.serialize_bytes(self.usermap.encode_to_vec().as_slice())
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

#[cfg(test)]
mod test {
  use tokio_util::bytes::Buf;

  use super::UserStore;

  fn ser_de(store: &UserStore) -> UserStore {
    let encoding = bincode::serialize(store).unwrap();
    bincode::deserialize(encoding.as_slice()).unwrap()
  }

  #[test]
  fn test_empty() {
    let store = UserStore::new();
    assert_eq!(store.num_users(), 0);
  }

  #[test]
  fn test_serde_empty() {
    let store = UserStore::new();
    let store = ser_de(&store);
    assert_eq!(store.num_users(), 0);
  }

  #[test]
  fn test_one_user() {
    let mut store = UserStore::new();
    store
      .add_user("bob".to_owned(), "bob's password".to_owned())
      .unwrap();
    assert_eq!(store.num_users(), 1);
    assert!(store.find_user("bob").is_some_and(|user| user
      .password
      .as_ref()
      .is_some_and(|password| password == "bob's password")));
  }

  #[test]
  fn test_serde_one_user() {
    let mut store = UserStore::new();
    store
      .add_user("bob".to_owned(), "bob's password".to_owned())
      .unwrap();
    let store = ser_de(&store);
    assert_eq!(store.num_users(), 1);
    assert!(store.find_user("bob").is_some_and(|user| user
      .password
      .as_ref()
      .is_some_and(|password| password == "bob's password")));
  }

  #[test]
  fn test_no_repeat_usernames() {
    let mut store = UserStore::new();
    store
      .add_user("bob".to_owned(), "bob's password".to_owned())
      .unwrap();
    store
      .add_user("bob".to_owned(), "new password".to_owned())
      .expect_err("Should not be allowed to add existing user");
    assert_eq!(store.num_users(), 1);
    assert!(store.find_user("bob").is_some_and(|user| user
      .password
      .as_ref()
      .is_some_and(|password| password == "bob's password")));
  }

  #[test]
  fn test_no_empty_username() {
    let mut store = UserStore::new();
    store
      .add_user("".to_owned(), "password".to_owned())
      .expect_err("Can't add a user with an empty username");
    assert_eq!(store.num_users(), 0);
  }

  #[test]
  fn test_two_users() {
    let mut store = UserStore::new();
    store
      .add_user(
        "bob vance vance refrigeration".to_owned(),
        "bob's cold".to_owned(),
      )
      .unwrap();
    store
      .add_user("joe".to_owned(), "bad password".to_owned())
      .unwrap();
    assert_eq!(store.num_users(), 2);
    assert!(store
      .find_user("bob vance vance refrigeration")
      .is_some_and(|user| user
        .password
        .as_ref()
        .is_some_and(|password| password == "bob's cold")));
    assert!(store.find_user("joe").is_some_and(|user| user
      .password
      .as_ref()
      .is_some_and(|password| password == "bad password")));
  }

  #[test]
  fn test_adjacent_serializations() {
    let mut store1 = UserStore::new();
    store1
      .add_user("joe".to_owned(), "bad password".to_owned())
      .unwrap();
    let mut store2 = UserStore::new();
    store2.add_user("a".to_owned(), "a".to_owned()).unwrap();
    store2.add_user("b".to_owned(), "b".to_owned()).unwrap();
    let mut encoding = bincode::serialize(&store1).unwrap();
    encoding.extend(bincode::serialize(&store2).unwrap());

    let mut reader = encoding.reader();
    let store1: UserStore = bincode::deserialize_from(&mut reader).unwrap();
    let store2: UserStore = bincode::deserialize_from(&mut reader).unwrap();
    assert_eq!(store1.num_users(), 1);
    assert_eq!(store2.num_users(), 2);
  }
}
