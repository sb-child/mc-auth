use axum::{routing, Json, Router};
use mc_auth::{models::meta::meta_resp, prisma};
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

  let app = Router::new()
    .route("/", routing::get(index))
    .route("/authserver/authenticate", routing::post(login))
    .layer(TraceLayer::new_for_http());

  let listener = TcpListener::bind("127.0.0.1:2345").await.unwrap();
  tracing::info!("认证服务器正在监听 {}", listener.local_addr().unwrap());

  axum::serve(listener, app).await.unwrap();
  Ok(())
}

async fn index() -> Json<meta_resp::GetMetadataResp> {
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

async fn login() -> Json<meta_resp::GetMetadataResp> {
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
