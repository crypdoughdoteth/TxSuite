#[cfg(test)]
mod tests {
    use anyhow::Result;
    use argon2::{
        password_hash::{rand_core::OsRng, SaltString},
        Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
    };
    use reqwest::Client;
    use serde_json::json;
    use shared_crypto::intent::{Intent, IntentMessage};
    use sui_keys::keypair_file::read_keypair_from_file;
    use sui_sdk::{
        types::programmable_transaction_builder::ProgrammableTransactionBuilder, SuiClientBuilder,
    };
    use sui_types::{
        base_types::SuiAddress,
        crypto::Signature,
        transaction::{
            Argument,
            Command::{self, SplitCoins},
            TransactionData,
        },
    };

    use crate::rpc::types::{JsonRpcRequest, JsonRpcResponse};
    use crate::{
        database::types::{ProjectDatabases, RELATIONAL_DATABASE},
        routes::{
            register::register_user,
            router::rpc_router,
            types::{ApiResult, SponsoredTxArgs, UserRegistration},
        },
    };
    use axum::{routing::post, Router};
    use tokio::net::TcpListener;
    use tracing_subscriber::fmt::format::FmtSpan;
    #[tokio::test]
    async fn sponsor() -> Result<()> {
        // will fail if sui balances of the provided keys are 0 on devnet
        // spawn server on new thread to mimic server
        tokio::spawn(async move {
            let app = Router::new()
                // `GET /rpc/v1` is our generalized JSON-RPC router
                // Contrary to standard, if you send me an array of args
                // in the response, I will return BAD_REQUEST, since that's what it is.
                // Fuck the standard.
                .route("/rpc/v1", post(rpc_router));
            // run our app with hyper, listening globally on port 3000
            let listener = TcpListener::bind("0.0.0.0:4200").await.unwrap();
            axum::serve(listener, app).await.unwrap();
        });

        let alice = read_keypair_from_file("../../testing/sponsoredtx/keys/alice.key")?;
        let bob = read_keypair_from_file("../../testing/sponsoredtx/keys/bob.key")?;

        let alice_address = SuiAddress::from(&alice.public());
        let bob_address = SuiAddress::from(&bob.public());
        println!("Alice: {}", &alice_address);
        println!("Bob: {}", &bob_address);

        let sui_devnet = SuiClientBuilder::default().build_devnet().await?;
        println!("Sui devnet version: {}", sui_devnet.api_version());

        let balances = sui_devnet
            .coin_read_api()
            .get_coins(alice_address, Some("0x2::sui::SUI".to_owned()), None, None)
            .await?;
        println!("{:?}", balances);
        let sponsor_coin = balances.data.into_iter().next().unwrap();
        let mut ptb = ProgrammableTransactionBuilder::new();
        let amt = ptb.pure(1000u64)?;
        ptb.command(SplitCoins(Argument::GasCoin, vec![amt]));
        let arg_addy = ptb.pure(alice_address)?;
        ptb.command(Command::TransferObjects(
            vec![Argument::Result(0)],
            arg_addy,
        ));
        let fin = ptb.finish();
        let td: TransactionData = TransactionData::new_programmable_allow_sponsor(
            bob_address,
            vec![sponsor_coin.object_ref()],
            fin,
            5_000_000u64,
            sui_devnet.read_api().get_reference_gas_price().await?,
            alice_address,
        );

        let intent = Intent::sui_transaction();
        let b_sig = Signature::new_secure(&IntentMessage::new(intent.clone(), &td), &bob);
        let res = Client::builder()
            .build()?
            .post("http://localhost:4200/rpc/v1")
            .json(&JsonRpcRequest {
                jsonrpc: "2.0".to_owned(),
                method: "sponsoredTx".to_owned(),
                params: Some(json!(SponsoredTxArgs {
                    signature: b_sig,
                    tx_data: td
                })),
                id: 1,
            })
            .send()
            .await?
            .json::<JsonRpcResponse>()
            .await?;

        println!("\n{:?}", res);

        Ok(())
    }

    #[test]
    fn hasher() {
        let salt = SaltString::generate(&mut OsRng);
        let hashed_pass = Argon2::default()
            .hash_password(b"reeeee", &salt)
            .unwrap()
            .to_string();
        let hash = PasswordHash::new(hashed_pass.as_str()).unwrap();
        let valid = Argon2::default().verify_password(b"reeeee", &hash).is_ok();
        assert!(valid);
        println!("{hash}");
    }

    #[tokio::test]
    async fn register() -> Result<()> {
        ProjectDatabases::init(Some(())).await.unwrap();
        tokio::spawn(async move {
            tracing_subscriber::fmt()
                .with_span_events(FmtSpan::CLOSE)
                .with_max_level(tracing::Level::INFO)
                .with_target(false)
                .init();

            let app = Router::new()
                // `GET /rpc/v1` is our generalized JSON-RPC router
                // Contrary to standard, if you send me an array of args
                // in the response, I will return BAD_REQUEST, since that's what it is.
                // Fuck the standard.
                .route("/api/register", post(register_user));
            // run our app with hyper, listening globally on port 3000
            let listener = TcpListener::bind("0.0.0.0:3001").await.unwrap();
            axum::serve(listener, app).await.unwrap();
        });
        println!("Server is running on port 3001");
        let client = reqwest::Client::builder()
            .build()?
            .post("http://localhost:3001/api/register")
            .json(&UserRegistration {
                email: String::from("testingemail@test.com"),
                password: String::from("reeeeee"),
            })
            .send()
            .await?
            .json::<ApiResult>()
            .await?;

        assert!(client.res.to_string().contains("testingemail@test.com") == true);

        sqlx::query!("DELETE FROM Users WHERE email = ?", "testingemail@test.com")
            .execute(RELATIONAL_DATABASE.get().unwrap())
            .await?;

        Ok(())
    }
}
