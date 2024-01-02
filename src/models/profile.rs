use base64::Engine;
use rsa::{
  pkcs1v15::SigningKey,
  signature::{RandomizedSigner, SignatureEncoding},
};
use serde::{Deserialize, Serialize};
use sha1::Sha1;

use super::textures::ProfileTextures;
use crate::{prisma, settings::Settings, utils};

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
    Self {
      id: utils::uuid_vec_to_string(data.uuid),
      name: data.display_name,
      properties: vec![Properties {
        name: "uploadableTextures".to_owned(),
        signature: None,
        value: match data.uploadable_textures {
          prisma::UploadableTextures::SkinOnly => "skin",
          prisma::UploadableTextures::SkinAndCape => "skin,cape",
          prisma::UploadableTextures::None => "",
        }
        .to_owned(),
      }],
    }
  }

  pub fn with_textures(self: Self, textures: ProfileTextures) -> Self {
    let mut x = self.clone();
    x.properties.push(Properties { name: "textures".to_owned(), signature: None, value: textures.to_base64() });
    x
  }

  pub fn with_settings(self: Self, sett: Settings) -> Self {
    let prikey = if let Some(prikey) = sett.signature.prikey_obj.clone() {
      prikey
    } else {
      return self;
    };
    let sign_key = SigningKey::<Sha1>::new(prikey);
    let mut rng = rand::thread_rng();
    let be = utils::base64();
    let x: Vec<Properties> = self
      .properties
      .iter()
      .map(|prop| {
        let mut p = prop.clone();
        let sign = sign_key.sign_with_rng(&mut rng, prop.value.as_bytes()).to_bytes();
        p.signature = Some(be.encode(sign));
        p
      })
      .collect();
    Profile { properties: x, ..self }
  }
}
