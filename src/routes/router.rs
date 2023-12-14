use super::types::SponsoredTxArgs;
use crate::rpc::types::{JsonRpcError, JsonRpcRequest, JsonRpcResponse, Methods};
use axum::Json;
use serde_json::{json, Value};
use tracing::log::info;

#[tracing::instrument]
pub async fn rpc_router(
    Json(payload): Json<JsonRpcRequest>,
) -> Result<Json<JsonRpcResponse>, JsonRpcError> {
    info!("Processing JSON-RPC Call ... ");
    let id = payload.id;
    let method = Methods::parse(payload.method.as_str());
    let args: Option<Value> = payload.params;

    match (method, args) {
        (Ok(Methods::SponsoredTx { call }), Some(a)) => {
            let arg = serde_json::from_value::<SponsoredTxArgs>(a).map_err(|e| {
                JsonRpcError::new(
                    -32602,
                    "Invalid params".to_owned(),
                    Some(json!(e.to_string())),
                )
            })?;
            Ok(Json(call(arg, id).await?))
        }
        (Err(e), _) => {
            let json_rpc_err: JsonRpcError = JsonRpcError::new(
                -32601,
                String::from("The method does not exist / is not available"),
                Some(json!(e.to_string())),
            );
            Err(json_rpc_err)
        }
        (_, _) => {
            Err(JsonRpcError::new(-32600, "Invalid Request".to_owned(), None))
        }
    }
}
