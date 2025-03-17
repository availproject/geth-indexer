use crate::types::ToHexString;
use alloy::{
    primitives::U256,
    rpc::types::eth::{Parity, Signature, Transaction as AlloyTransaction},
};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(
    Clone,
    Debug,
    Default,
    Queryable,
    Selectable,
    Insertable,
    Identifiable,
    QueryableByName,
    Serialize,
    Deserialize,
    PartialEq,
)]
#[diesel(primary_key(chain_id))]
#[diesel(table_name = crate::schema::chains)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Chain {
    pub chain_id: i64,
    pub latest_tps: i64,
}
#[derive(
    Clone,
    Debug,
    Queryable,
    Selectable,
    Insertable,
    Identifiable,
    Associations,
    Serialize,
    Deserialize,
)]
#[diesel(primary_key(chain_id, transaction_hash))]
#[diesel(belongs_to(Chain, foreign_key = chain_id))]
#[diesel(table_name = crate::schema::transactions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct TransactionModel {
    pub chain_id: i64,
    pub transaction_hash: String,
    pub transaction_nonce: String,
    pub block_hash: Option<String>,
    pub block_number: Option<i64>,
    pub transaction_index: Option<i64>,
    pub _from: String,
    pub _to: Option<String>,
    pub value: String,
    pub gas_price: Option<String>,
    pub gas: String,
    pub input: String,
    pub v: String,
    pub r: String,
    pub s: String,
    pub transaction_type: String,
    pub impersonated: bool,
    pub max_priority_fee_per_gas: Option<String>,
    pub max_fee_per_gas: Option<String>,
}

impl From<TransactionModel> for AlloyTransaction {
    fn from(value: TransactionModel) -> Self {
        let v = value.v.parse().unwrap();
        let signature = Some(Signature {
            r: value.r.parse().unwrap(),
            s: value.s.parse().unwrap(),
            v,
            y_parity: if v.to::<u64>() < 2 {
                Some(Parity(v.to()))
            } else {
                None
            },
        });
        Self {
            hash: value.transaction_hash.parse().unwrap(),
            nonce: value
                .transaction_nonce
                .parse::<U256>()
                .unwrap_or_default()
                .to(),
            block_hash: value.block_hash.map(|x| x.parse().unwrap()),
            block_number: Some(value.block_number.unwrap() as u64),
            transaction_index: Some(value.transaction_index.unwrap() as u64),
            from: value._from.parse().unwrap(),
            to: value._to.map(|x| x.parse().unwrap()),
            value: value.value.parse().unwrap(),
            gas_price: value.gas_price.map(|x| x.parse::<U256>().unwrap().to()),
            gas: value.gas.parse::<U256>().unwrap().to(),
            input: value.input.parse().unwrap(),
            transaction_type: if value.transaction_type.is_empty() {
                None
            } else {
                Some(value.transaction_type.parse::<U256>().unwrap().to())
            },
            access_list: Some(Default::default()),
            max_priority_fee_per_gas: value
                .max_priority_fee_per_gas
                .map(|x| x.parse::<U256>().unwrap().to()),
            max_fee_per_gas: value
                .max_fee_per_gas
                .map(|x| x.parse::<U256>().unwrap().to()),
            chain_id: Some(value.chain_id as u64),
            signature,
            ..Default::default()
        }
    }
}

impl TransactionModel {
    pub fn from(chain_id: u64, value: &AlloyTransaction) -> Self {
        let result = Self {
            chain_id: chain_id.try_into().unwrap(),
            transaction_hash: value.hash.to_hex_string(),
            transaction_nonce: value.nonce.to_hex_string(),
            block_hash: value.block_hash.map(|x| x.to_hex_string()),
            block_number: value.block_number.map(|x| x as i64),
            transaction_index: value.transaction_index.map(|x| x as i64),
            _from: value.from.to_hex_string(),
            _to: value.to.map(|x| x.to_hex_string()),
            value: value.value.to_hex_string(),
            gas_price: value.gas_price.map(|x| x.to_hex_string()),
            gas: value.gas.to_hex_string(),
            input: value.input.to_hex_string(),
            v: value
                .signature
                .map(|sign| sign.v.to_hex_string())
                .unwrap_or_default(),
            r: value
                .signature
                .map(|sign| sign.r.to_hex_string())
                .unwrap_or_default(),
            s: value
                .signature
                .map(|sign| sign.s.to_hex_string())
                .unwrap_or_default(),
            transaction_type: value
                .transaction_type
                .map(|x| x.to_hex_string())
                .unwrap_or_default(),
            impersonated: false,
            max_priority_fee_per_gas: value.max_priority_fee_per_gas.map(|x| x.to_hex_string()),
            max_fee_per_gas: value.max_fee_per_gas.map(|x| x.to_hex_string()),
        };
        #[cfg(debug_assertions)]
        {
            let mut converted_back: AlloyTransaction = result.clone().into();
            converted_back.access_list = value.access_list.clone();
            assert_eq!(*value, converted_back);
        }
        result
    }
}
