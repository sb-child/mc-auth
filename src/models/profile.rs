use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Properties {
  #[serde(rename = "name")]
  pub name: String,

  #[serde(rename = "signature")]
  pub signature: Option<String>,

  #[serde(rename = "value")]
  pub value: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Profile {
  #[serde(rename = "id")]
  pub id: String,

  #[serde(rename = "name")]
  pub name: String,

  #[serde(rename = "properties")]
  pub properties: Vec<Properties>,
}
