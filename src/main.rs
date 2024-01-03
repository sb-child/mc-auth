use std::sync::Arc;

use axum::{extract::State, routing, Json, Router};
use mc_auth::{
  app_state::AppState,
  models::{
    error, login as login_model,
    meta::meta_resp,
    profile::{self, Profile},
    refresh as refresh_model, textures,
    user::{self, User},
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

  tracing_subscriber::registry()
    .with(tracing_subscriber::fmt::layer())
    // .with(LevelFilter::DEBUG)
    .with(tracing_subscriber::filter::filter_fn(|metadata| {
      // println!("{}", metadata.target());
      (
        (metadata.target().starts_with("tokio_postgres::") ||
         metadata.target().starts_with("sql_schema_")) && metadata.level() <= &LevelFilter::WARN)
        || (metadata.target().starts_with("mc_auth") && metadata.level() <= &LevelFilter::DEBUG)
        || (metadata.target().starts_with("tower_http::") && metadata.level() <= &LevelFilter::DEBUG)
    }))
    .init();

  tracing::info!("色麦块认证服务器~");

  let settings_str = match fs::read_to_string("Settings.toml").await {
    Ok(v) => v,
    Err(_e) => "".to_owned(),
  };
  let mut settings: Settings = toml::from_str(&settings_str)?;
  settings.signature = settings.signature.convert();

  let webserver_settings = settings.web_server.clone();

  // tracing::debug!("配置: {:?}", settings);

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
    // API 元数据获取
    .route("/", routing::get(index))
    // 登录
    .route("/authserver/authenticate", routing::post(login))
    // 刷新
    .route("/authserver/refresh", routing::post(login))
    // 验证令牌
    .route("/authserver/validate", routing::post(login))
    // 吊销令牌
    .route("/authserver/invalidate", routing::post(login))
    // 登出
    .route("/authserver/signout", routing::post(login))
    // 客户端进入服务器
    .route("/sessionserver/session/minecraft/join", routing::post(login))
    // 服务端验证客户端
    .route("/sessionserver/session/minecraft/hasJoined", routing::get(login))
    // 查询角色属性
    .route("/sessionserver/session/minecraft/profile/:uuid", routing::get(login))
    // 按名称批量查询角色
    .route("/api/profiles/minecraft", routing::get(login))
    // 上传材质
    .route("/api/user/profile/:uuid/:textureType", routing::put(login))
    // 清除材质
    .route("/api/user/profile/:uuid/:textureType", routing::delete(login))
    // 获取材质
    .route("/textures/:hash", routing::get(login))
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
    signature_publickey: state.settings.signature.pubkey,
  })
}

async fn login(
  State(state): State<AppState>,
  req: Json<login_model::req::LoginReq>,
) -> Result<Json<login_model::resp::LoginResp>, error::ErrorResponse> {
  let access_token = utils::gen_access_token();
  let client_token = req.client_token.clone().unwrap_or(utils::gen_uuid());
  let request_user = match req.request_user {
    Some(x) => x,
    None => false,
  };
  let default_max_tokens = state.settings.token.max;
  let default_token_need_refresh_duration = state.settings.token.refresh_duration;
  let default_token_invalid_duration = state.settings.token.invalid_duration;
  let user: Result<(Option<prisma::profile::Data>, prisma::user::Data), login_model::LoginTransactionError> = state
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
        Err(login_model::LoginTransactionError::InvalidUser)
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
        login_model::LoginTransactionError::InvalidUser | login_model::LoginTransactionError::WrongPassword => {
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
          vec![user.1.id],
        )
        .await
      }
    })
    .await;

  if let Err(e) = check_tokens_result {
    tracing::debug!("刷新令牌失败: {:?}", e);
    return Err(error::Error::new_database_error().to_response());
  }

  tracing::debug!("请求: {:?}", req);

  let profiles = user.1.profile().unwrap();
  let profiles = profiles
    .iter()
    .map(|x| {
      profile::Profile::from_query(x.clone())
        .with_textures(textures::ProfileTextures::from_query(x.clone()).with_settings(state.settings.clone()))
        .with_settings(state.settings.clone())
    })
    .collect();
  let user_info = user::User { id: utils::uuid_vec_to_string(user.1.uuid), properties: vec![] };
  tracing::debug!("角色列表: {:?}", profiles);
  Ok(Json(login_model::resp::LoginResp {
    access_token,
    client_token,
    available_profiles: profiles,
    selected_profile: user.0.map_or_else(
      || None,
      |x| {
        Some(
          profile::Profile::from_query(x.clone())
            .with_textures(textures::ProfileTextures::from_query(x.clone()).with_settings(state.settings.clone()))
            .with_settings(state.settings.clone()),
        )
      },
    ),
    user: request_user.then(|| user_info),
  }))
}

async fn refresh(
  State(state): State<AppState>,
  req: Json<refresh_model::req::RefreshReq>,
) -> Result<Json<refresh_model::resp::RefreshResp>, error::ErrorResponse> {
  let access_token = req.access_token.clone();
  let client_token = req.client_token.clone();
  let request_user = match req.request_user {
    Some(x) => x,
    None => false,
  };
  let selected_profile = match req.selected_profile {
    Some(x) => Some(utils::string_to_uuid_vec(x.id)),
    None => None,
  };
  let default_max_tokens = state.settings.token.max;
  let default_token_need_refresh_duration = state.settings.token.refresh_duration;
  let default_token_invalid_duration = state.settings.token.invalid_duration;
  let data: Result<
    (Option<prisma::profile::Data>, prisma::user::Data, String, String),
    refresh_model::RefreshTransactionError,
  > = state
    .db
    ._transaction()
    .run(|cli| {
      let access_token = access_token.clone();
      let client_token = client_token.clone();
      let selected_profile = selected_profile.clone();
      async move {
        let token = utils::get_token(cli, access_token, client_token).await?;
        let (user, profile, token_client_token) = match token {
          Some(x) => (*x.owner().unwrap(), x.profile().unwrap(), x.client_token),
          None => {
            return Err(refresh_model::RefreshTransactionError::InvalidToken);
          },
        };
        match utils::check_tokens(
          cli,
          default_max_tokens,
          default_token_need_refresh_duration,
          default_token_invalid_duration,
          vec![user.id],
        )
        .await
        {
          Ok(_) => {},
          Err(err) => {
            return Err(refresh_model::RefreshTransactionError::QueryError(err));
          },
        };
        let token = utils::get_token(cli, access_token, client_token).await?;
        let (user, profile, token_client_token) = match token {
          Some(x) => (*x.owner().unwrap(), x.profile().unwrap(), x.client_token),
          None => {
            return Err(refresh_model::RefreshTransactionError::InvalidToken);
          },
        };
        let s_profile = match selected_profile {
          Some(x) => {
            let p = cli
              .profile()
              .find_unique(prisma::profile::UniqueWhereParam::UuidEquals(x))
              .with(prisma::profile::skin::fetch())
              .with(prisma::profile::cape::fetch())
              .exec()
              .await;
            match p {
              Ok(Some(pd)) => Some(pd),
              Ok(None) => None,
              Err(err) => {
                return Err(refresh_model::RefreshTransactionError::QueryError(err));
              },
            }
          },
          None => None,
        };
        let profile = match s_profile {
          Some(x) => {
            match profile {
              Some(_y) => {
                return Err(refresh_model::RefreshTransactionError::ReassignProfile);
              },
              None => Some(x),
            }
          },
          None => profile.cloned(),
        };
        let client_token = match client_token {
          Some(x) => x,
          None => token_client_token,
        };
        let add_token_result = utils::add_token(
          cli,
          match profile {
            Some(x) => Some(x.id),
            None => None,
          },
          user.id,
          access_token,
          client_token,
        )
        .await;
        match add_token_result {
          Ok(_) => {},
          Err(err) => {
            return Err(refresh_model::RefreshTransactionError::QueryError(err));
          },
        };
        Ok((profile, user, access_token, client_token))
      }
    })
    .await;
  let (profile, user, access_token, client_token) = match data {
    Ok(x) => (x.0, x.1, x.2, x.3),
    Err(err) => {
      match err {
        refresh_model::RefreshTransactionError::QueryError(err) => {
          return Err(error::Error::new_database_error().to_response());
        },
        refresh_model::RefreshTransactionError::InvalidToken => {
          return Err(error::Error::new_invalid_token().to_response());
        },
        refresh_model::RefreshTransactionError::ReassignProfile => {
          return Err(error::Error::new_reassign_profile().to_response());
        },
      }
    },
  };
  Ok(Json(refresh_model::resp::RefreshResp {
    access_token,
    client_token,
    selected_profile: match profile {
      Some(x) => {
        Some(
          Profile::from_query(x.clone())
            .with_textures(textures::ProfileTextures::from_query(x).with_settings(state.settings.clone()))
            .with_settings(state.settings),
        )
      },
      None => None,
    },
    user: request_user.then(|| User { id: utils::uuid_vec_to_string(user.uuid), properties: vec![] }),
  }))
}
