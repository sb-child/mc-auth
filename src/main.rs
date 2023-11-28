use axum::{routing, Json, Router};
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Serialize, Deserialize)]
struct MetaLinks {
    homepage: String,
    register: String,
}

#[derive(Serialize, Deserialize)]
struct Meta {
    #[serde(rename = "serverName")]
    server_name: String,
    #[serde(rename = "implementationName")]
    implementation_name: String,
    #[serde(rename = "implementationVersion")]
    implementation_version: String,
    links: MetaLinks,
}

#[derive(Serialize, Deserialize)]
struct GetMetadataResp {
    meta: Meta,
    #[serde(rename = "skinDomains")]
    skin_domains: Vec<String>,
    #[serde(rename = "signaturePublickey")]
    signature_publickey: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();

    let app = Router::new()
        .route("/", routing::get(index))
        .layer(TraceLayer::new_for_http());

    let listener = TcpListener::bind("127.0.0.1:2345").await.unwrap();
    tracing::debug!("认证服务端正在监听 {}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.unwrap();
    Ok(())
}

async fn index() -> Json<GetMetadataResp> {
    Json(GetMetadataResp {
        meta: Meta {
            server_name: "色麦块".to_owned(),
            implementation_name: "色麦块认证服务器".to_owned(),
            implementation_version: "0.0.1".to_owned(),
            links: MetaLinks {
                homepage: "https://sbchild.top/".to_owned(),
                register: "https://sbchild.top/".to_owned(),
            },
        },
        skin_domains: vec!["sbchild.top".to_owned(), "*.sbchild.top".to_owned()],
        signature_publickey: "".to_owned(),
    })
}
