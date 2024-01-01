use rsa::{
  pkcs8::{DecodePrivateKey, DecodePublicKey, EncodePrivateKey, EncodePublicKey, LineEnding},
  RsaPrivateKey, RsaPublicKey,
};
use serde::Deserialize;

fn default_server_name() -> String {
  "认证服务器".to_string()
}

fn default_implementation_name() -> String {
  env!("CARGO_PKG_NAME").to_string()
}

fn default_implementation_version() -> String {
  env!("CARGO_PKG_VERSION").to_string()
}

fn default_skin_domains() -> Vec<String> {
  vec![]
}

fn default_homepage_link() -> String {
  "https://github.com/sb-child/".to_owned()
}

fn default_register_link() -> String {
  "https://github.com/sb-child/".to_owned()
}

fn default_prikey() -> String {
  "".to_owned()
}

fn default_token_max() -> i64 {
  10
}

fn default_token_refresh() -> i64 {
  // 15 天
  1296000
}

fn default_token_invalid() -> i64 {
  // 5 天
  432000
}

fn default_webserver_listen() -> String {
  "127.0.0.1:2345".to_owned()
}

fn default_textures_base() -> String {
  "http://127.0.0.1:2345/textures/".to_owned()
}

fn default_textures_max_size() -> u64 {
  // max 16 MB
  2 * 1024
}

fn default_textures_max_length() -> u64 {
  // max 512 pixels in height or width
  512
}

#[derive(Deserialize, Debug, Clone, Default)]
pub struct Token {
  #[serde(rename = "max", default = "default_token_max")]
  pub max: i64,

  #[serde(rename = "refresh", default = "default_token_refresh")]
  pub refresh_duration: i64,

  #[serde(rename = "invalid", default = "default_token_invalid")]
  pub invalid_duration: i64,
}

#[derive(Deserialize, Debug, Clone, Default)]
pub struct WebServer {
  #[serde(rename = "listen", default = "default_webserver_listen")]
  pub listen: String,
}

#[derive(Deserialize, Debug, Clone, Default)]
pub struct Signature {
  #[serde(skip)]
  pub pubkey: String,

  #[serde(skip)]
  pub pubkey_obj: Option<RsaPublicKey>,

  #[serde(rename = "prikey", default = "default_prikey")]
  pub prikey: String,

  #[serde(skip)]
  pub prikey_obj: Option<RsaPrivateKey>,
}

impl Signature {
  pub fn convert(self: Self) -> Self {
    let private_key = RsaPrivateKey::from_pkcs8_pem(&self.prikey);
    let keypair = match private_key {
      Ok(mut k) => {
        k.precompute().unwrap();
        let prikey_str = k.to_pkcs8_pem(LineEnding::default()).unwrap().to_string();
        let pubkey = k.to_public_key();
        let pubkey_str = pubkey.to_public_key_pem(LineEnding::default()).unwrap();
        Some((k, prikey_str, pubkey, pubkey_str))
      },
      Err(err) => {
        tracing::warn!("签名私钥解析失败, 将禁用签名机制: {:?}", err);
        None
      },
    };
    if let Some(keypair) = keypair {
      Self { prikey: keypair.1, pubkey: keypair.3, prikey_obj: Some(keypair.0), pubkey_obj: Some(keypair.2) }
    } else {
      Self { ..Default::default() }
    }
  }
}

#[derive(Deserialize, Debug, Clone, Default)]
pub struct Textures {
  #[serde(rename = "base", default = "default_textures_base")]
  pub base: String,

  #[serde(rename = "max-size-kb", default = "default_textures_max_size")]
  pub max_size: u64,

  #[serde(rename = "max-length", default = "default_textures_max_length")]
  pub max_length: u64,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Settings {
  #[serde(rename = "server-name", default = "default_server_name")]
  pub server_name: String,

  #[serde(rename = "implementation-name", default = "default_implementation_name")]
  pub implementation_name: String,

  #[serde(rename = "implementation-version", default = "default_implementation_version")]
  pub implementation_version: String,

  #[serde(rename = "skin-domains", default = "default_skin_domains")]
  pub skin_domains: Vec<String>,

  #[serde(rename = "homepage-link", default = "default_homepage_link")]
  pub homepage_link: String,

  #[serde(rename = "register-link", default = "default_register_link")]
  pub register_link: String,

  #[serde(rename = "token")]
  pub token: Token,

  #[serde(rename = "signature")]
  pub signature: Signature,

  #[serde(rename = "webserver")]
  pub web_server: WebServer,

  #[serde(rename = "textures")]
  pub textures: Textures,
}
