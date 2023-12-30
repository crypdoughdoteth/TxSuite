mod database;
mod routes;
mod rpc;
mod tests;
use axum::{routing::post, Router};
use dotenvy::dotenv;
use routes::{router::rpc_router, register::register_user};
use tokio::net::TcpListener;
use tracing::log::info;
use tracing_subscriber::fmt::format::FmtSpan;
use crate::database::types::ProjectDatabases; 
#[tokio::main]
async fn main() {
    // initialize tracing
    dotenv().unwrap();
    tracing_subscriber::fmt()
        .with_span_events(FmtSpan::CLOSE)
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .init();
    info!("Starting TxSuite ... ");
    ProjectDatabases::init(None).await.unwrap();
    let app = Router::new()
        // `GET /rpc/v1` is our generalized JSON-RPC router
        // Contrary to standard, if you send me an array of args
        // in the response, I will return BAD_REQUEST, since that's what it is.
        // Fuck the standard.
        .route("/rpc/v1", post(rpc_router))
        .route("/api/register", post(register_user));
    // run our app with hyper, listening globally on port 3000
    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
    info!("TxSuite is running on port 3000");
    axum::serve(listener, app).await.unwrap();
}
