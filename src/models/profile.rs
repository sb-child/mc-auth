use serde::{Deserialize, Serialize};

use crate::{prisma, utils};

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

impl Profile {
  pub fn from_query(data: prisma::profile::Data) -> Self {
    Self { id: utils::uuid_vec_to_string(data.uuid), name: data.display_name, properties: vec![] }
  }
}
