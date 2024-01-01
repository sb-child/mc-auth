use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Error {
  #[serde(rename = "cause")]
  pub cause: Option<String>,

  #[serde(rename = "error")]
  pub error: String,

  #[serde(rename = "errorMessage")]
  pub error_message: String,

  #[serde(skip)]
  pub status_code: axum::http::StatusCode,
}

pub type ErrorResponse = (axum::http::StatusCode, axum::Json<Error>);

impl Error {
  /// 数据库错误 (自定义)
  pub fn new_database_error() -> Self {
    Self {
      cause: None,
      error: "Service Unavailable".to_owned(),
      error_message: "Database operation error.".to_owned(),
      status_code: axum::http::StatusCode::SERVICE_UNAVAILABLE,
    }
  }

  /// 令牌无效
  pub fn new_invalid_token() -> Self {
    Self {
      cause: None,
      error: "ForbiddenOperationException".to_owned(),
      error_message: "Invalid token.".to_owned(),
      status_code: axum::http::StatusCode::FORBIDDEN,
    }
  }

  /// 密码错误，或短时间内多次登录失败而被暂时禁止登录
  pub fn new_invalid_credentials() -> Self {
    Self {
      cause: None,
      error: "ForbiddenOperationException".to_owned(),
      error_message: "Invalid credentials. Invalid username or password.".to_owned(),
      status_code: axum::http::StatusCode::FORBIDDEN,
    }
  }

  /// 试图向一个已经绑定了角色的令牌指定其要绑定的角色
  pub fn new_reassign_profile() -> Self {
    Self {
      cause: None,
      error: "IllegalArgumentException".to_owned(),
      error_message: "Access token already has a profile assigned.".to_owned(),
      status_code: axum::http::StatusCode::BAD_REQUEST,
    }
  }

  /// 试图向一个令牌绑定不属于其对应用户的角色 (非标准)
  pub fn new_assign_others_profile() -> Self {
    Self {
      cause: None,
      error: "ForbiddenOperationException".to_owned(),
      error_message: "Profile not found.".to_owned(),
      status_code: axum::http::StatusCode::FORBIDDEN,
    }
  }

  /// 试图使用一个错误的角色加入服务器
  pub fn new_invalid_profile() -> Self {
    Self {
      cause: None,
      error: "ForbiddenOperationException".to_owned(),
      error_message: "Invalid token.".to_owned(),
      status_code: axum::http::StatusCode::FORBIDDEN,
    }
  }

  pub fn to_response(self) -> ErrorResponse {
    (self.status_code, axum::Json::from(self))
  }
}
