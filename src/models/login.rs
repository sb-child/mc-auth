pub mod login_req {
  use serde::{Deserialize, Serialize};

  #[derive(Serialize, Deserialize, Debug, Clone)]
  pub struct Agent {
    #[serde(rename = "name")]
    pub name: Option<String>,

    #[serde(rename = "version")]
    pub version: Option<i32>,
  }

  #[derive(Serialize, Deserialize, Debug, Clone)]
  pub struct LoginReq {
    #[serde(rename = "agent")]
    pub agent: Option<Agent>,

    #[serde(rename = "clientToken")]
    pub client_token: Option<String>,

    #[serde(rename = "password")]
    pub password: String,

    #[serde(rename = "requestUser")]
    pub request_user: Option<bool>,

    #[serde(rename = "username")]
    pub username: String,
  }
}

pub mod login_resp {
  use serde::{Deserialize, Serialize};

  // todo
  #[derive(Serialize, Deserialize, Debug, Clone)]
  pub struct Profile {}

  // todo
  #[derive(Serialize, Deserialize, Debug, Clone)]
  pub struct User {}

  #[derive(Serialize, Deserialize, Debug, Clone)]
  pub struct LoginResp {
    #[serde(rename = "accessToken")]
    pub access_token: String,

    #[serde(rename = "availableProfiles")]
    pub available_profiles: Vec<Profile>,

    #[serde(rename = "clientToken")]
    pub client_token: String,

    #[serde(rename = "selectedProfile")]
    pub selected_profile: Option<Profile>,

    #[serde(rename = "user")]
    pub user: Option<User>,
  }
}

#[derive(thiserror::Error, Debug)]
pub enum LoginTransactionError {
  #[error("数据库错误: {0}")]
  QueryError(#[from] prisma_client_rust::QueryError),
  #[error("用户不存在")]
  InvalidUser,
  #[error("密码不正确")]
  WrongPassword,
}
