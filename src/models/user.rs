use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Properties {
  #[serde(rename = "name")]
  pub name: String,

  #[serde(rename = "value")]
  pub value: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct User {
  #[serde(rename = "id")]
  pub id: String,

  #[serde(rename = "properties")]
  pub properties: Vec<Properties>,
}
