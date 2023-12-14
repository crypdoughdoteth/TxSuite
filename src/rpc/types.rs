use serde::{Serialize, Deserialize}; 
use serde_json::{Value, json};
use crate::routes::types::SponsoredTxArgs; 
use std::{future::Future, pin::Pin};
use axum::{response::{IntoResponse, Response}, http::StatusCode};
use std::fmt::Formatter;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub method: String,
    pub params: Option<Value>,
    pub id: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub result: Option<JsonRpcResult>,
    pub error: Option<JsonRpcError>, 
    pub id: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JsonRpcError {
    pub code: i32, 
    pub message: String,
    pub data: Option<Value>,
}

impl std::fmt::Display for JsonRpcError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {        
        write!(f, "\nerror code: {}, \nmessage: {}, \ndata: {:#?}", self.code, self.message, self.data) 
    } 
}

impl std::error::Error for JsonRpcError {}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum JsonRpcResult {
    SponsoredTxResult(String)
}

type AsyncFn = Box<dyn FnOnce(SponsoredTxArgs, u32) -> Pin<Box<dyn Future<Output = Result<JsonRpcResponse, JsonRpcError>> + Send>> + Send>;

pub enum Methods {    
    SponsoredTx{
        call: AsyncFn,  
    },
}

impl JsonRpcResponse {
    pub fn new(result: Option<JsonRpcResult>, error: Option<JsonRpcError>, id: u32) -> Self {
        Self {
            jsonrpc: "2.0".to_owned(),
            result,  
            error,
            id
        }
    }
}

impl JsonRpcError {
    pub fn new(code: i32, message: String, data: Option<Value>) -> Self {
        Self {
            code, 
            message, 
            data,
            
        }
    }
}

impl IntoResponse for JsonRpcError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            JsonRpcError::new(-32603, "Internal Error".to_owned(), Some(json!(self.data)))
        )
            .into_response()
    }
}

impl IntoResponse for JsonRpcResponse {
    fn into_response(self) -> Response {
        (
            StatusCode::OK,
            self
        )
            .into_response()
    }
}
