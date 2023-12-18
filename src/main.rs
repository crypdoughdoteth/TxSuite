mod rpc;
mod routes;
mod tls;
use axum::{
    routing::post,
    Router
};
use tracing_subscriber::fmt::format::FmtSpan;
use routes::router::rpc_router; 
use tracing::log::info; 
use tokio::net::TcpListener;
use openssl::ssl::SslAcceptor;
use tls::{server::TlsServer, config::Config}; 

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt()
        .with_span_events(FmtSpan::CLOSE)
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .init();
    info!("Starting TxSuite ... ");
   
    let tls_acceptor: SslAcceptor = Config::init(); 

   let app = Router::new()
        // `GET /rpc/v1` is our generalized JSON-RPC router
        // Contrary to standard, if you send me an array of args
        // in the response, I will return BAD_REQUEST, since that's what it is. 
        // Fuck the standard.
        .route("/rpc/v1", post(rpc_router));
    // run our app with hyper, listening globally on port 3000
    let tcp_listener = TcpListener::bind("0.0.0.0:3000").await.unwrap(); 
    info!("TxSuite is running on port 3000"); 
    TlsServer::serve(tcp_listener, tls_acceptor, app).await.unwrap();  
}


