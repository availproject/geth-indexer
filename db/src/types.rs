use std::fmt;

use alloy::{
    primitives::{hex::ToHexExt, Address, Bloom, Bytes, FixedBytes, Log as PrimitiveLog, U256, U64},
    rpc::types::{eth::Transaction, TransactionReceipt},
    signers::k256::ecdsa::SigningKey,
    sol,
    sol_types::SolEvent,
};

use chrono::{DateTime, FixedOffset, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};


#[derive(Clone)]
pub enum Metric {
    CurrentTPS,
    LiveTPS,
    TransactionVolume,
    TotalTransactions,
    SuccessfulTransfers,
}

#[derive(Serialize, Deserialize)]
pub struct TxResponse {
    pub successful_txns: u64,
    pub total_txns: u64,
    pub timestamp: String,
}

impl std::str::FromStr for Metric {
    type Err = serde_json::Error;

    fn from_str(input: &str) -> Result<Metric, Self::Err> {
        match input {
            "current_tps" => Ok(Metric::CurrentTPS),
            "live_tps" => Ok(Metric::LiveTPS),
            "transaction_volume" => Ok(Metric::TransactionVolume),
            "total_transfers" => Ok(Metric::TotalTransactions),
            "successful_transfers" => Ok(Metric::SuccessfulTransfers),
            _ => Ok(Metric::CurrentTPS),
        }
    }
}

pub trait IfSomeBase {
    fn if_some(&self) -> usize;
    fn _extract_is_some(&self) -> bool;
}
impl<T> IfSomeBase for Option<T> {
    fn if_some(&self) -> usize {
        self.is_some() as usize
    }
    fn _extract_is_some(&self) -> bool {
        self.is_some()
    }
}

macro_rules! extract {
    (
        $(#[$attr:meta])*
        pub struct $name:ident {
            $($(#[$field_attr:meta])*
            pub $field:ident : $t:ty),+}) => {
        $(#[$attr])*
        pub struct $name { $(pub $field: $t),+ }
        impl $name {
            #[allow(dead_code)]
            pub fn field_count(&self) -> usize {
                #[allow(unused_imports)]
                use $crate::types::{IfSomeBase};
                extract!(@count self $($field,)*)
            }
            #[allow(dead_code)]
            pub fn field_name(&self) -> &'static str {
                let mut result = "";
                $(
                    if (&self.$field)._extract_is_some() {
                        result = stringify!($field);
                    }
                )*
                result
            }
        }
    };
    (@count $self:ident $first_field:ident, $($rest:ident,)*) => {
        (&$self.$first_field).if_some()
            + extract!(@count $self $($rest,)*)
    };
    (@count $self:ident) => { 0 };
}

extract! {
    #[derive(Deserialize, Serialize)]
    pub struct Order {
        pub order: Option<String>
    }
}

extract! {
    #[derive(Deserialize, Serialize)]
    pub struct ChainId {
        pub chain_id: Option<u64>
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub struct Stride {
    pub stride: Option<u64>,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct Type {
    pub tx_type: Option<Tx>,
}

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
pub enum Tx {
    Native,
    CrossChain,
}

impl std::str::FromStr for Tx {
    type Err = serde_json::Error;

    fn from_str(input: &str) -> Result<Tx, Self::Err> {
        match input {
            "native" => Ok(Tx::Native),
            "cross_chain" => Ok(Tx::CrossChain),
            _ => Ok(Tx::Native),
        }
    }
}

impl fmt::Display for Tx {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Tx::Native => "native",
            Tx::CrossChain => "cross_chain",
        };
        write!(f, "{}", s)
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub struct Limit {
    pub limit: Option<u64>,
}

extract! {
    #[derive(Clone, Deserialize, Serialize)]
    pub struct TxFilter {
        pub sender: Option<String>,
        pub status: Option<i16>,
        pub recipient: Option<String>,
        pub chain_id: Option<u64>
    }
}

extract! {
    #[derive(Deserialize, Serialize)]
    pub struct TxIdentifier {
        pub tx_hash: Option<String>,
        pub latest: Option<bool>,
        pub page_idx: Option<u64>
    }
}

extract! {
    #[derive(Deserialize, Serialize)]
    pub struct Parts {
        pub all: Option<bool>,
        pub summary_only: Option<bool>
    }
}

#[derive(Deserialize, Serialize)]
pub struct TxnSummary {
    pub hash: String,
    pub block_hash: Option<String>,
    pub to: Option<String>,
    pub from: String,
    pub status: Option<u8>,
    pub value: String,
    pub block_height: u64,
}

impl From<Transaction> for TxnSummary {
    fn from(tx: Transaction) -> Self {
        TxnSummary {
            hash: tx.hash.to_hex_string(),
            block_hash: Some(tx.block_hash.unwrap().to_hex_string()),
            to: if tx.to.is_none() {
                None
            } else {
                Some(tx.to.unwrap().to_hex_string())
            },
            from: tx.from.to_hex_string(),
            status: Some(1),
            value: tx.value.to_hex_string(),
            block_height: tx.block_number.unwrap(),
        }
    }
}

#[derive(Deserialize, Serialize)]
pub enum TxAPIResponse {
    TxnSummary(TxnSummary),
    Transaction(Transaction),
}

sol! {
    #[derive(Debug)]
    event ETHReceivedFromSourceChainInBatch(
        uint32 indexed sourceChainId,
        address[] recipients,
        uint256[] amounts,
        uint32 startMessageId,
        uint32 indexed endMessageId
    );
}

pub fn parse_logs(receipt: &TransactionReceipt) -> (bool, u32) {
    let signature_bytes: FixedBytes<32> = ETHReceivedFromSourceChainInBatch::SIGNATURE_HASH;
    for log in receipt.inner.logs() {
        let primitive_log = PrimitiveLog {
            address: log.address(),
            data: log.data().clone(),
        };

        if log.topics()[0] == signature_bytes {
            if let Ok(event) = ETHReceivedFromSourceChainInBatch::decode_log(&primitive_log, false)
            {
                tracing::info!("Start Message ID: {:?}", event.startMessageId);
                tracing::info!("End Message ID: {:?}", event.endMessageId);
                return (true, event.endMessageId - event.startMessageId);
            } else {
                return (false, 0);
            }
        }
    }

    return (false, 0);
}

pub trait ToHexString {
    fn to_hex_string(&self) -> String;
}

impl ToHexString for &[u8] {
    fn to_hex_string(&self) -> String {
        to_hex_string_internal(self)
    }
}

impl ToHexString for Address {
    fn to_hex_string(&self) -> String {
        to_hex_string_internal(self.as_slice())
    }
}

impl<const N: usize> ToHexString for FixedBytes<N> {
    fn to_hex_string(&self) -> String {
        to_hex_string_internal(self.as_slice())
    }
}

impl ToHexString for U64 {
    fn to_hex_string(&self) -> String {
        to_hex_string_internal(self.to_be_bytes::<8>().as_slice())
    }
}

impl ToHexString for Transaction {
    fn to_hex_string(&self) -> String {
        self.hash.to_hex_string()
    }
}

impl ToHexString for U256 {
    fn to_hex_string(&self) -> String {
        to_hex_string_internal(self.to_be_bytes::<32>().as_slice())
    }
}

impl ToHexString for u8 {
    fn to_hex_string(&self) -> String {
        U256::from(*self).to_hex_string()
    }
}

impl ToHexString for u64 {
    fn to_hex_string(&self) -> String {
        U256::from(*self).to_hex_string()
    }
}

impl ToHexString for u128 {
    fn to_hex_string(&self) -> String {
        U256::from(*self).to_hex_string()
    }
}

impl ToHexString for bool {
    fn to_hex_string(&self) -> String {
        U256::from(*self).to_hex_string()
    }
}

impl ToHexString for Bytes {
    fn to_hex_string(&self) -> String {
        to_hex_string_internal(&self.0)
    }
}

impl ToHexString for Bloom {
    fn to_hex_string(&self) -> String {
        self.as_slice().to_hex_string()
    }
}

impl ToHexString for SigningKey {
    fn to_hex_string(&self) -> String {
        self.to_bytes().encode_hex_with_prefix()
    }
}

fn to_hex_string_internal(bytes: &[u8]) -> String {
    bytes.encode_hex_with_prefix()
}

#[warn(dead_code)]
pub fn unix_ms_to_ist(timestamp: i64) -> String {
    let timestamp_ms = if timestamp < 1_000_000_000_000 {
        timestamp * 1000
    } else {
        timestamp
    };

    let secs = timestamp_ms / 1000;
    let nanos = (timestamp_ms % 1000) * 1_000_000;
    let naive_dt =
        NaiveDateTime::from_timestamp_opt(secs, nanos as u32).expect("Invalid timestamp");
    let utc_dt = DateTime::<Utc>::from_naive_utc_and_offset(naive_dt, Utc);
    let ist_offset = FixedOffset::east_opt(5 * 3600 + 30 * 60).expect("Invalid offset");
    let ist_dt = utc_dt.with_timezone(&ist_offset);

    ist_dt.format("%Y-%m-%d %H:%M:%S%.3f IST").to_string()
}

pub const MAX_WINDOW_SIZE: u64 = 25;
