mod rpc;
mod routes;
use axum::{
    routing::get,
    Router, extract::Request
};
use tracing_subscriber::fmt::format::FmtSpan;
use routes::router::rpc_router; 
use tracing::log::{info, error, warn}; 
use futures_util::pin_mut;

use hyper_util::rt::{TokioExecutor, TokioIo};
use tokio_openssl::SslStream;
use tower::Service;
use tokio::net::TcpListener;
use std::{path::PathBuf, pin::Pin};
use openssl::ssl::{Ssl, SslAcceptor, SslFiletype, SslMethod};
use hyper::body::Incoming;

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt()
        .with_span_events(FmtSpan::CLOSE)
        .with_max_level(tracing::Level::DEBUG)
        .with_target(false)
        .init();
    info!("Starting TxSuite ... ");
   
    let mut tls_builder = SslAcceptor::mozilla_modern_v5(SslMethod::tls()).unwrap();

    tls_builder
        .set_certificate_file(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("self_signed_certs")
                .join("ca.pem"),
            SslFiletype::PEM,
        )
        .unwrap();

    tls_builder
        .set_private_key_file(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("self_signed_certs")
                .join("ca-key.pem"),
            SslFiletype::PEM,
        )
        .unwrap();

    tls_builder.check_private_key().unwrap();

    let tls_acceptor = tls_builder.build();

   let app = Router::new()
        // `GET /rpc/v1` is our generalized JSON-RPC router
        // Contrary to standard, if you send me an array of args
        // in the response, I will return BAD_REQUEST, since that's what it is. 
        // Fuck the standard.
        .route("/rpc/v1", get(rpc_router));
    // run our app with hyper, listening globally on port 3000
    let tcp_listener = TcpListener::bind("[::1]:3000").await.unwrap(); 
    info!("TxSuite is running on port 3000");
    // axum::serve(listener, app).await.unwrap();

    pin_mut!(tcp_listener);

    loop {
        let tower_service = app.clone();
        let tls_acceptor = tls_acceptor.clone();

        // Wait for new tcp connection
        let (cnx, addr) = tcp_listener.accept().await.unwrap();

        tokio::spawn(async move {
            let ssl = Ssl::new(tls_acceptor.context()).unwrap();
            let mut tls_stream = SslStream::new(ssl, cnx).unwrap();
            if let Err(err) = SslStream::accept(Pin::new(&mut tls_stream)).await {
                error!(
                    "error during tls handshake connection from {}: {}",
                    addr, err
                );
                return;
            }

            // Hyper has its own `AsyncRead` and `AsyncWrite` traits and doesn't use tokio.
            // `TokioIo` converts between them.
            let stream = TokioIo::new(tls_stream);

            // Hyper has also its own `Service` trait and doesn't use tower. We can use
            // `hyper::service::service_fn` to create a hyper `Service` that calls our app through
            // `tower::Service::call`.
            let hyper_service = hyper::service::service_fn(move |request: Request<Incoming>| {
                // We have to clone `tower_service` because hyper's `Service` uses `&self` whereas
                // tower's `Service` requires `&mut self`.
                //
                // We don't need to call `poll_ready` since `Router` is always ready.
                tower_service.clone().call(request)
            });

            let ret = hyper_util::server::conn::auto::Builder::new(TokioExecutor::new())
                .serve_connection_with_upgrades(stream, hyper_service)
                .await;

            if let Err(err) = ret {
                warn!("error serving connection from {}: {}", addr, err);
            }
        });
    }

}
