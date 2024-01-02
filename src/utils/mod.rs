use prisma::PrismaClient;
use rand::Rng;

use crate::prisma;

pub fn gen_access_token() -> String {
  let mut rng = rand::thread_rng();
  let characters: Vec<char> = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789".chars().collect();
  let token: String = (0..32)
    .map(|_| {
      let idx = rng.gen_range(0..characters.len());
      characters[idx] as char
    })
    .collect();
  token
}

pub fn gen_uuid() -> String {
  uuid::Uuid::new_v4().as_simple().to_string()
}

pub fn uuid_vec_to_string(x: Vec<u8>) -> String {
  uuid::Uuid::from_slice(&x).unwrap().as_simple().to_string()
}

pub fn string_to_uuid_vec(x: String) -> Vec<u8> {
  match uuid::Uuid::parse_str(&x) {
    Ok(x) => x.as_bytes().to_vec(),
    Err(_err) => {
      vec![]
    },
  }
}

pub async fn get_token(
  cli: PrismaClient,
  access_token: String,
  client_token: Option<String>,
) -> Result<Option<prisma::token::Data>, prisma_client_rust::QueryError> {
  match client_token {
    Some(client_token) => {
      cli
        .token()
        .find_first(vec![
          prisma::token::WhereParam::AccessToken(prisma::read_filters::StringFilter::Equals(access_token)),
          prisma::token::WhereParam::ClientToken(prisma::read_filters::StringFilter::Equals(client_token)),
        ])
        .with(prisma::token::owner::fetch())
        .with(prisma::token::profile::fetch().with(prisma::profile::skin::fetch()).with(prisma::profile::cape::fetch()))
    },
    None => {
      cli.token().find_first(vec![prisma::token::WhereParam::AccessToken(prisma::read_filters::StringFilter::Equals(
        access_token,
      ))])
    },
  }
  .exec()
  .await
}

pub async fn del_token(
  cli: PrismaClient,
  access_token: String,
) -> Result<prisma::token::Data, prisma_client_rust::QueryError> {
  cli.token().delete(prisma::token::UniqueWhereParam::AccessTokenEquals(access_token)).exec().await
}

pub async fn add_token(
  cli: PrismaClient,
  profile: Option<i64>,
  user: i64,
  access_token: String,
  client_token: String,
) -> Result<prisma::token::Data, prisma_client_rust::QueryError> {
  if let Some(profile_id) = profile {
    cli
      .token()
      .create(access_token, client_token, prisma::user::UniqueWhereParam::IdEquals(user), vec![
        prisma::token::SetParam::ConnectProfile(prisma::profile::UniqueWhereParam::IdEquals(profile_id)),
      ])
      .exec()
  } else {
    cli.token().create(access_token, client_token, prisma::user::UniqueWhereParam::IdEquals(user), vec![]).exec()
  }
  .await
}

pub async fn check_tokens(
  cli: PrismaClient,
  default_max_tokens: i64,
  default_token_need_refresh_duration: i64,
  default_token_invalid_duration: i64,
  user_id: Vec<i64>,
) -> Result<(), prisma_client_rust::QueryError> {
  let results = cli
    .user()
    .find_many(
      user_id.iter().map(|x| prisma::user::WhereParam::Id(prisma::read_filters::BigIntFilter::Equals(*x))).collect(),
    )
    .exec()
    .await?
    .into_iter()
    .map(|u| {
      (
        // 可用
        cli
          .token()
          .find_many(vec![
            prisma::token::status::equals(prisma::TokenStatus::Available),
            prisma::token::owner::is(vec![prisma::user::WhereParam::Id(prisma::read_filters::BigIntFilter::Equals(
              u.id,
            ))]),
          ])
          .order_by(prisma::token::created_at::order(prisma::SortOrder::Desc)),
        // 需要刷新
        cli
          .token()
          .find_many(vec![
            prisma::token::status::equals(prisma::TokenStatus::NeedRefresh),
            prisma::token::owner::is(vec![prisma::user::WhereParam::Id(prisma::read_filters::BigIntFilter::Equals(
              u.id,
            ))]),
          ])
          .order_by(prisma::token::created_at::order(prisma::SortOrder::Desc)),
        // 用户设置
        cli.setting().find_unique(prisma::setting::UniqueWhereParam::UserIdEquals(u.id)),
      )
    });
  let results = cli._batch(results).await?;
  let now = chrono::Utc::now();
  for (_, (avaliable, need_refresh, settings)) in results.into_iter().enumerate() {
    let (max_tokens, token_need_refresh_duration, token_invalid_duration) = if let Some(item) = settings {
      (item.max_token, item.token_need_refresh_duration, item.token_invalid_duration)
    } else {
      (default_max_tokens, default_token_need_refresh_duration, default_token_invalid_duration)
    };
    let mut token_counter = max_tokens;
    // 如果超过创建时间 + token_need_refresh_duration, 则设置令牌为 NeedRefresh
    // 如果 token_counter <= 0, 则设置令牌为 Invalid
    for (_, x) in avaliable.into_iter().enumerate() {
      tracing::info!("检查用户 {} 的可用令牌 {}...", x.owner_id, x.id);
      if {
        token_counter -= 1;
        token_counter > 0
      } {
        if now.timestamp() - x.created_at.timestamp() > token_need_refresh_duration {
          tracing::info!("用户 {} 的可用令牌 {} 暂时失效", x.owner_id, x.id);
          cli
            .token()
            .update(prisma::token::UniqueWhereParam::IdEquals(x.id), vec![prisma::token::SetParam::Status(
              prisma::write_params::TokenStatusParam::Set(prisma::TokenStatus::NeedRefresh),
            )])
            .exec()
            .await?;
        }
      } else {
        tracing::info!("用户 {} 的可用令牌 {} 因数量超出而永久失效", x.owner_id, x.id);
        cli
          .token()
          .update(prisma::token::UniqueWhereParam::IdEquals(x.id), vec![prisma::token::SetParam::Status(
            prisma::write_params::TokenStatusParam::Set(prisma::TokenStatus::Invalid),
          )])
          .exec()
          .await?;
      }
    }
    // 如果超过创建时间 + token_need_refresh_duration + invalid_duration, 则设置令牌为 Invalid
    for (_, x) in need_refresh.into_iter().enumerate() {
      tracing::info!("检查用户 {} 的暂时失效令牌 {}...", x.owner_id, x.id);
      if now.timestamp() - x.created_at.timestamp() > token_need_refresh_duration + token_invalid_duration {
        tracing::info!("用户 {} 的暂时失效令牌 {} 永久失效", x.owner_id, x.id);
        cli
          .token()
          .update(prisma::token::UniqueWhereParam::IdEquals(x.id), vec![prisma::token::SetParam::Status(
            prisma::write_params::TokenStatusParam::Set(prisma::TokenStatus::Invalid),
          )])
          .exec()
          .await?;
      }
    }
  }
  return Ok(());
}

pub fn texture_vec_to_string(x: Vec<u8>) -> String {
  x.iter().map(|byte| format!("{:02x}", byte)).collect()
}

pub fn base64() -> base64::engine::general_purpose::GeneralPurpose {
  base64::engine::GeneralPurpose::new(&base64::alphabet::STANDARD, base64::engine::GeneralPurposeConfig::new())
}
