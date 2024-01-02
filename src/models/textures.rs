use base64::Engine as _;
use serde::{Deserialize, Serialize};

use crate::{prisma, settings::Settings, utils};

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

impl ProfileTextures {
  pub fn from_query(data: prisma::profile::Data) -> Self {
    let cloned_data = data.clone();
    let skin_data = cloned_data.skin();
    let cape_data = cloned_data.cape();
    let now = chrono::Utc::now().timestamp_millis() as u64;
    Self {
      timestamp: now,
      profile_id: utils::uuid_vec_to_string(data.uuid),
      profile_name: data.display_name,
      textures: Textures {
        cape: if let Ok(Some(texture)) = cape_data {
          Some(Texture { metadata: Metadata { model: None }, url: utils::texture_vec_to_string(texture.hash.clone()) })
        } else {
          None
        },
        skin: if let Ok(Some(texture)) = skin_data {
          Some(Texture {
            metadata: Metadata {
              model: Some(
                match texture.model {
                  prisma::SkinType::Default => "default",
                  prisma::SkinType::Slim => "slim",
                }
                .to_owned(),
              ),
            },
            url: utils::texture_vec_to_string(texture.hash.clone()),
          })
        } else {
          None
        },
      },
    }
  }

  pub fn with_settings(self: Self, sett: Settings) -> Self {
    let base_url = &sett.textures.base;
    let skin = if let Some(ref skin) = self.textures.skin {
      let mut skin = skin.clone();
      skin.url = base_url.to_owned() + &skin.url;
      Some(skin)
    } else {
      None
    };
    let cape = if let Some(ref cape) = self.textures.cape {
      let mut cape = cape.clone();
      cape.url = base_url.to_owned() + &cape.url;
      Some(cape)
    } else {
      None
    };
    Self { textures: Textures { cape, skin }, ..self }
  }

  pub fn to_base64(self: Self) -> String {
    let be = utils::base64();
    be.encode(match self.serialize(serde_json::value::Serializer) {
      Ok(v) => v.to_string(),
      Err(_err) => "".to_owned(),
    })
  }
}
