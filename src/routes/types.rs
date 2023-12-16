use sui_types::{crypto::Signature, transaction::TransactionData};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct SponsoredTxArgs {
    pub signature: Signature,
    pub tx_data: TransactionData,
}
