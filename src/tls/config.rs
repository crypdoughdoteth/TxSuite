use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
use std::path::PathBuf; 
pub struct Config;

impl Config {
    pub fn init() -> SslAcceptor {
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

        tls_builder.build()
    }
}
