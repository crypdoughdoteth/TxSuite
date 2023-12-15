mod rpc;
mod routes;
mod tls;
use axum::{
    routing::get,
    Router
};
use tracing_subscriber::fmt::format::FmtSpan;
use routes::router::rpc_router; 
use tracing::log::info; 

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt()
        .with_span_events(FmtSpan::CLOSE)
        .with_max_level(tracing::Level::DEBUG)
        .with_target(false)
        .init();
    info!("Starting TxSuite ... ");
    // build our application with a route
    let app = Router::new()
        // `GET /rpc/v1` is our generalized JSON-RPC router
        // Contrary to standard, if you send me an array of args
        // in the response, I will return BAD_REQUEST, since that's what it is. 
        // Fuck the standard. 
        .route("/rpc/v1", get(rpc_router));
    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap(); 
    info!("TxSuite is running on port 3000");
    axum::serve(listener, app).await.unwrap();
}
