use std::sync::Arc;

use axum::{extract::State, routing, Json, Router};
use mc_auth::{
  app_state::AppState,
  models::{
    error,
    login::{login_req, login_resp, LoginTransactionError},
    meta::meta_resp,
    profile, user,
  },
  prisma,
  settings::Settings,
  utils,
};
use prisma::PrismaClient;
use prisma_client_rust::NewClientError;
use tokio::{fs, net::TcpListener};
use tower_http::trace::TraceLayer;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  // color_backtrace::install();
  // better_panic::Settings::debug().most_recent_first(false).lineno_suffix(true).install();

  tracing_subscriber::registry().with(tracing_subscriber::fmt::layer()).with(LevelFilter::INFO).init();

  tracing::info!("色麦块认证服务器~");

  let settings_str = match fs::read_to_string("Settings.toml").await {
    Ok(v) => v,
    Err(_e) => "".to_owned(),
  };
  let settings: Settings = toml::from_str(&settings_str)?;

  let webserver_settings = settings.web_server.clone();

  tracing::debug!("配置: {:?}", settings);

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

  // db.user()
  //   .create(
  //     vec![0, 1, 2, 3, 4, 5, 6, 7, 8],
  //     "sbchild".to_owned(),
  //     "sbchild0@gmail.com".to_owned(),
  //     "password".to_owned(),
  //     vec![],
  //   )
  //   .exec()
  //   .await?;

  // db.user()
  //   .update(prisma::user::UniqueWhereParam::EmailEquals("sbchild0@gmail.com".to_owned()), vec![
  //     prisma::user::SetParam::SetUuid(uuid::Uuid::new_v4().as_bytes().to_vec()),
  //   ])
  //   .exec()
  //   .await?;

  // db.profile()
  //   .update(prisma::profile::UniqueWhereParam::DisplayNameEquals("sb-child".to_owned()), vec![
  //     prisma::profile::SetParam::SetUuid(uuid::Uuid::new_v4().as_bytes().to_vec()),
  //   ])
  //   .exec()
  //   .await?;

  // db.profile().create(
  //   vec![1, 2, 3, 4, 5, 6, 7, 8, 9],
  //   prisma::user::email::equals("sbchild0@gmail.com".to_owned()),
  //   "sb-child".to_owned(),
  //   vec![],
  // ).exec().await?;

  let state = AppState { db: Arc::new(db), settings };

  let app = Router::new()
    .route("/", routing::get(index))
    .route("/authserver/authenticate", routing::post(login))
    .with_state(state)
    .layer(TraceLayer::new_for_http());

  let listener = TcpListener::bind(webserver_settings.listen).await?;
  tracing::info!("web服务器正在监听 {}", listener.local_addr().unwrap());

  axum::serve(listener, app).await?;
  Ok(())
}

async fn index(State(state): State<AppState>) -> Json<meta_resp::GetMetadataResp> {
  Json(meta_resp::GetMetadataResp {
    meta: meta_resp::Meta {
      server_name: state.settings.server_name,
      implementation_name: state.settings.implementation_name,
      implementation_version: state.settings.implementation_version,
      links: meta_resp::MetaLinks { homepage: state.settings.homepage_link, register: state.settings.register_link },
    },
    skin_domains: state.settings.skin_domains,
    signature_publickey: state.settings.pubkey,
  })
}

async fn login(
  State(state): State<AppState>,
  req: Json<login_req::LoginReq>,
) -> Result<Json<login_resp::LoginResp>, error::ErrorResponse> {
  let access_token = utils::gen_access_token();
  let client_token = req.client_token.clone().unwrap_or(utils::gen_uuid());
  let default_max_tokens = state.settings.token.max;
  let default_token_need_refresh_duration = state.settings.token.refresh_duration;
  let default_token_invalid_duration = state.settings.token.invalid_duration;
  let user: Result<(Option<prisma::profile::Data>, prisma::user::Data), LoginTransactionError> = state
    .db
    ._transaction()
    .run(|cli| {
      let req = req.clone();
      let access_token = access_token.clone();
      let client_token = client_token.clone();
      async move {
        let include_display_name = req.username.split_once(":");
        match include_display_name {
          Some((dn, email)) => {
            // 根据 角色名+邮箱 匹配用户
            let user_match_displayname = cli
              .user()
              .find_first(vec![
                prisma::user::profile::every(vec![prisma::profile::display_name::equals(dn.to_string())]),
                prisma::user::email::equals(email.to_string()),
                prisma::user::password::equals(req.password.clone()),
              ])
              .with(
                prisma::user::profile::fetch(vec![])
                  .with(prisma::profile::skin::fetch())
                  .with(prisma::profile::cape::fetch()),
              )
              .exec()
              .await?;
            if let Some(user) = user_match_displayname {
              let profile = cli
                .profile()
                .find_unique(prisma::profile::UniqueWhereParam::DisplayNameEquals(dn.to_string()))
                .with(prisma::profile::skin::fetch())
                .with(prisma::profile::cape::fetch())
                .exec()
                .await?
                .unwrap();
              utils::add_token(cli, Some(profile.id), user.id, access_token, client_token).await?;
              return Ok((Some(profile), user));
            }
          },
          None => {
            // 根据邮箱匹配用户
            let user_match_email = cli
              .user()
              .find_first(vec![
                prisma::user::email::equals(req.username.clone()),
                prisma::user::password::equals(req.password.clone()),
              ])
              .with(
                prisma::user::profile::fetch(vec![])
                  .with(prisma::profile::skin::fetch())
                  .with(prisma::profile::cape::fetch()),
              )
              .exec()
              .await?;
            if let Some(user) = user_match_email {
              utils::add_token(cli, None, user.id, access_token, client_token).await?;
              return Ok((None, user));
            }
          },
        }
        // 如果找不到用户, 则返回错误
        Err(LoginTransactionError::InvalidUser)
      }
    })
    .await;

  let user = match user {
    Ok(v) => {
      tracing::debug!("匹配到用户 {:?}", v);
      v
    },
    Err(e) => {
      tracing::debug!("登录失败: {:?}", e);
      match e {
        LoginTransactionError::InvalidUser | LoginTransactionError::WrongPassword => {
          return Err(error::Error::new_invalid_credentials().to_response());
        },
        _ => {
          return Err(error::Error::new_database_error().to_response());
        },
      }
    },
  };

  let check_tokens_result = state
    .db
    ._transaction()
    .run(|cli| {
      async move {
        utils::check_tokens(
          cli,
          default_max_tokens,
          default_token_need_refresh_duration,
          default_token_invalid_duration,
        )
        .await
      }
    })
    .await;

  if let Err(e) = check_tokens_result {
    return Err(error::Error::new_database_error().to_response());
  }

  tracing::debug!("请求: {:?}", req);

  let profiles = user.1.profile().unwrap();
  let profiles = profiles
    .iter()
    .map(|x| {
      profile::Profile {
        id: utils::uuid_vec_to_string(x.uuid.clone()),
        name: x.display_name.clone(),
        properties: vec![],
      }
    })
    .collect();
  let user_info = user::User { id: utils::uuid_vec_to_string(user.1.uuid), properties: vec![] };
  tracing::debug!("角色列表: {:?}", profiles);
  Ok(Json(login_resp::LoginResp {
    access_token,
    client_token,
    available_profiles: profiles,
    selected_profile: user.0.map_or_else(
      || None,
      |x| Some(profile::Profile { id: utils::uuid_vec_to_string(x.uuid), name: x.display_name, properties: vec![] }),
    ),
    user: Some(user_info),
  }))
  // Json(login_resp::LoginResp {
  //   access_token,
  //   available_profiles: vec![profile::Profile {
  //     id: "67f0d17981804a03ad851dbd6bbd4eb8".to_owned(),
  //     name: "涩妹妹".to_owned(),
  //     properties: vec![],
  //   }],
  //   client_token,
  //   selected_profile: Some(profile::Profile {
  //     id: "67f0d17981804a03ad851dbd6bbd4eb8".to_owned(),
  //     name: "涩妹妹".to_owned(),
  //     properties: vec![],
  //   }),
  //   user: None,
  // })
  // Err(error::Error::new_invalid_credentials().to_response())
}
