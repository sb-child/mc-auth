use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Metadata {
  #[serde(rename = "model")]
  pub model: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Texture {
  #[serde(rename = "metadata")]
  pub metadata: Metadata,

  #[serde(rename = "url")]
  pub url: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Textures {
  #[serde(rename = "cape")]
  pub cape: Option<Texture>,

  #[serde(rename = "skin")]
  pub skin: Option<Texture>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProfileTextures {
  #[serde(rename = "profileId")]
  pub profile_id: String,

  #[serde(rename = "profileName")]
  pub profile_name: String,

  #[serde(rename = "textures")]
  pub textures: Textures,

  #[serde(rename = "timestamp")]
  pub timestamp: u64,
}
