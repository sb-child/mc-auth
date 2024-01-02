pub mod req {
  use serde::{Deserialize, Serialize};

  use crate::models::profile::Profile;

  #[derive(Serialize, Deserialize, Debug, Clone)]
  pub struct RefreshReq {
    #[serde(rename = "accessToken")]
    pub access_token: String,

    #[serde(rename = "clientToken")]
    pub client_token: Option<String>,

    #[serde(rename = "requestUser")]
    pub request_user: Option<bool>,

    #[serde(rename = "selectedProfile")]
    pub selected_profile: Option<Profile>,
  }
}

pub mod resp {
  use serde::{Deserialize, Serialize};

  use crate::models::{profile::Profile, user::User};

  #[derive(Serialize, Deserialize, Debug, Clone)]
  pub struct RefreshResp {
    #[serde(rename = "accessToken")]
    pub access_token: String,

    #[serde(rename = "clientToken")]
    pub client_token: String,

    #[serde(rename = "selectedProfile")]
    pub selected_profile: Option<Profile>,

    #[serde(rename = "user")]
    pub user: Option<User>,
  }
}

#[derive(thiserror::Error, Debug)]
pub enum RefreshTransactionError {
  #[error("数据库错误: {0}")]
  QueryError(#[from] prisma_client_rust::QueryError),
  #[error("令牌不存在")]
  InvalidToken,
  #[error("角色被重新绑定")]
  ReassignProfile,
}
