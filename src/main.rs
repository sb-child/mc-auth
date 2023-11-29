use std::sync::Arc;

use axum::{extract::State, routing, Json, Router};
use mc_auth::{
  app_state::AppState,
  models::{
    login::{login_req, login_resp, LoginTransactionError},
    meta::meta_resp,
  },
  prisma,
};
use prisma::PrismaClient;
use prisma_client_rust::NewClientError;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  tracing_subscriber::registry().with(tracing_subscriber::fmt::layer()).with(LevelFilter::INFO).init();

  tracing::info!("色麦块认证服务器~");
  tracing::info!("正在连接数据库...");
  let db: Result<PrismaClient, NewClientError> = PrismaClient::_builder().build().await;
  let db = match db {
    Ok(v) => v,
    Err(e) => {
      tracing::error!("无法连接到数据库: {}", e);
      return Err(anyhow::Error::new(e));
    },
  };
  tracing::info!("正在合并数据库...");
  match db._db_push().await {
    Ok(v) => {
      tracing::info!("合并了 {} 个对象, 数据库已经是最新的啦", v);
    },
    Err(e) => {
      tracing::error!("数据库合并失败: {}", e);
      return Err(anyhow::Error::new(e));
    },
  };

  // db.user().create(
  //   vec![0, 1, 2, 3, 4, 5, 6, 7, 8],
  //   "sbchild".to_owned(),
  //   "sbchild0@gmail.com".to_owned(),
  //   "password".to_owned(),
  //   vec![],
  // ).exec().await?;

  // db.profile().create(
  //   vec![1, 2, 3, 4, 5, 6, 7, 8, 9],
  //   prisma::user::email::equals("sbchild0@gmail.com".to_owned()),
  //   "sb-child".to_owned(),
  //   vec![],
  // ).exec().await?;

  let state = AppState { db: Arc::new(db) };

  let app = Router::new()
    .route("/", routing::get(index))
    .route("/authserver/authenticate", routing::post(login))
    .with_state(state)
    .layer(TraceLayer::new_for_http());

  let listener = TcpListener::bind("127.0.0.1:2345").await.unwrap();
  tracing::info!("认证服务器正在监听 {}", listener.local_addr().unwrap());

  axum::serve(listener, app).await.unwrap();
  Ok(())
}

async fn index(State(state): State<AppState>) -> Json<meta_resp::GetMetadataResp> {
  Json(meta_resp::GetMetadataResp {
    meta: meta_resp::Meta {
      server_name: "色麦块".to_owned(),
      implementation_name: "色麦块认证服务器".to_owned(),
      implementation_version: "0.0.1".to_owned(),
      links: meta_resp::MetaLinks {
        homepage: "https://sbchild.top/".to_owned(),
        register: "https://sbchild.top/".to_owned(),
      },
    },
    skin_domains: vec!["sbchild.top".to_owned(), "*.sbchild.top".to_owned()],
    signature_publickey: "".to_owned(),
  })
}

async fn login(State(state): State<AppState>, req: Json<login_req::LoginReq>) -> Json<login_resp::LoginResp> {
  tracing::info!("{:?}", state.db);
  let user: Result<mc_auth::prisma::user::Data, LoginTransactionError> = state
    .db
    ._transaction()
    .run(|cli| {
      let req = req.clone();
      async move {
        // 根据邮箱匹配用户
        let user_match_email = cli
          .user()
          .find_first(vec![
            prisma::user::email::equals(req.username.clone()),
            prisma::user::password::equals(req.password.clone()),
          ])
          .exec()
          .await?;
        if let Some(user) = user_match_email {
          return Ok(user);
        }
        // 根据游戏内名称匹配用户
        let user_match_displayname = cli
          .user()
          .find_first(vec![
            prisma::user::profile::some(vec![prisma::profile::display_name::equals(req.username.clone())]),
            prisma::user::password::equals(req.password.clone()),
          ])
          .exec()
          .await?;
        if let Some(user) = user_match_displayname {
          return Ok(user);
        }
        // 如果找不到用户, 则返回错误
        Err(LoginTransactionError::InvalidUser)
      }
    })
    .await;
  match user {
    Ok(v) => {
      tracing::info!("匹配到用户 {}", v.id);
    },
    Err(e) => {
      tracing::error!("登录失败: {:?}", e);
    },
  };
  tracing::info!("{:?}", req);
  Json(login_resp::LoginResp {
    access_token: "".to_owned(),
    available_profiles: vec![],
    client_token: "".to_owned(),
    selected_profile: None,
    user: None,
  })
}
