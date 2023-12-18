use axum::{Router, extract::Request};
use tokio::net::TcpListener;
use futures_util::pin_mut;
use openssl::ssl::{Ssl, SslAcceptor};
use tokio_openssl::SslStream;
use tower::Service; 
use hyper_util::rt::{TokioExecutor, TokioIo};
use tracing::log::{error, warn};
use std::pin::Pin; 
use hyper::body::Incoming;

pub struct TlsServer;

impl TlsServer {
    pub async fn serve(
        tcp_listener: TcpListener,
        tls_acceptor: SslAcceptor,
        app: Router,
    ) -> Result<(), Box<dyn std::error::Error>> {
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
                let hyper_service =
                    hyper::service::service_fn(move |request: Request<Incoming>| {
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
}
