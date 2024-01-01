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

fn default_pubkey() -> String {
  "".to_owned()
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

  #[serde(rename = "pubkey", default = "default_pubkey")]
  pub pubkey: String,

  #[serde(rename = "prikey", default = "default_prikey")]
  pub prikey: String,

  #[serde(rename = "token")]
  pub token: Token,

  #[serde(rename = "webserver")]
  pub web_server: WebServer,
}
