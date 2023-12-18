use super::types::SponsoredTxArgs;
use crate::rpc::types::{JsonRpcError, JsonRpcRequest, JsonRpcResponse, Methods};
use axum::Json;
use serde_json::Value;
use tracing::log::info;

pub async fn rpc_router(
    Json(payload): Json<JsonRpcRequest>,
) -> Result<Json<JsonRpcResponse>, JsonRpcError> {
    info!("Processing JSON-RPC Call ... ");
    let id = payload.id;
    let method = Methods::parse(payload.method.as_str());
    let args: Option<Value> = payload.params;

    match (method, args) {
        (Ok(Methods::SponsoredTx { call }), Some(a)) => {
            let arg = serde_json::from_value::<SponsoredTxArgs>(a)?;
            Ok(Json(call(arg, id).await?))
        }
        (Err(e), _) => Err(e),

        (_, _) => {
            Err(JsonRpcError::new(-32600, "Invalid Request".to_owned(), None))
        }
    }
}
