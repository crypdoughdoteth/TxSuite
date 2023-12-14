use serde_json::json;
use crate::routes::sponsor::sponsor_tx;
use super::types::{Methods, JsonRpcError}; 

impl Methods {

    pub fn parse(input: &str) -> Result<Methods, JsonRpcError> {

        match input {
            "sponsoredTx" => {
                Ok(Methods::SponsoredTx{call: 
                   Box::new(move |args, id| Box::pin(sponsor_tx(args, id)))
                })
            },
            _ => {
                Err(JsonRpcError::new(-32601, "Method not found".to_owned(), Some(json!("The method does not exist / is not available"))))
            }
            
        }
    }

}
