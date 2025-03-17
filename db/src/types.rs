use alloy::primitives::{hex::ToHexExt, Address, Bloom, Bytes, FixedBytes, U256, U64};
use alloy::rpc::types::eth::Transaction;
use alloy::signers::k256::ecdsa::SigningKey;
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
pub struct Limit {
    pub limit: Option<u64>,
}

extract! {
    #[derive(Clone, Deserialize, Serialize)]
    pub struct TxFilter {
        pub sender: Option<String>,
        pub status: Option<i16>,
        pub recipient: Option<String>
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
