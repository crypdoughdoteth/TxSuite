use crate::rpc::types::{JsonRpcResponse, JsonRpcResult, JsonRpcError};
use crate::routes::types::SponsoredTxArgs;
use serde_json::json;
use sui_types::{crypto::Signature, transaction::Transaction, quorum_driver_types::ExecuteTransactionRequestType}; 
use sui_keys::keypair_file::read_keypair_from_file;
use shared_crypto::intent::{Intent, IntentMessage};
use sui_types::signature::GenericSignature; 
use sui_sdk::{SuiClientBuilder, rpc_types::SuiTransactionBlockResponseOptions};
use tracing::{info, debug};

#[tracing::instrument(ret)]
pub async fn sponsor_tx(body: SponsoredTxArgs, id: u32) -> Result<JsonRpcResponse, JsonRpcError> {
    info!("Sponsored tx pending ... ");
    debug!("sponsoring tx: {:?}", &body);
    let devnet = SuiClientBuilder::default().build_devnet().await.map_err(|e| JsonRpcError::new(-32603, "Internal error".to_owned(), Some(json!(e.to_string()))))?;
    let key = read_keypair_from_file("../../../../testing/sponsoredtx/keys/alice.key").map_err(|e| JsonRpcError::new(-32603, "Internal error".to_owned(), Some(json!(e.to_string()))))?; 
    let intent = Intent::sui_transaction();
    let td = body.tx_data;
    let customer_signature = body.signature;
    let our_sig = Signature::new_secure(&IntentMessage::new(intent, &td), &key);
    let sigs = vec![GenericSignature::Signature(customer_signature), GenericSignature::Signature(our_sig)];
    let tx = Transaction::from_generic_sig_data(td, sigs);
    devnet.quorum_driver_api()
        .execute_transaction_block(tx, SuiTransactionBlockResponseOptions::full_content(), 
            Some(ExecuteTransactionRequestType::WaitForLocalExecution))
        .await.map_err(|e| JsonRpcError::new(-32603, "Internal error".to_owned(), Some(json!(e.to_string()))))?;
    info!("Sponsored tx successful!");
    Ok(JsonRpcResponse::new(Some(JsonRpcResult::SponsoredTxResult("Nice".to_owned())), None, id)) 
}
