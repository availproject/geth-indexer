use crate::types::ToHexString;
use alloy::{
    primitives::U256,
    rpc::types::eth::{
        Block as AlloyBlock, BlockTransactions, Header, Log, Parity, Receipt, ReceiptEnvelope,
        ReceiptWithBloom, Signature, Transaction as AlloyTransaction,
        TransactionReceipt as AlloyReceipt,
    },
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
#[diesel(primary_key(chain_id, block_number))]
#[diesel(belongs_to(Chain, foreign_key = chain_id))]
#[diesel(table_name = crate::schema::blocks)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct BlockModel {
    pub chain_id: i64,
    pub block_number: i64,
    pub block_hash: String,
    pub parent_hash: String,
    pub ommers_hash: String,
    pub beneficiary: String,
    pub state_root: String,
    pub transactions_root: String,
    pub receipts_root: String,
    pub logs_bloom: String,
    pub difficulty: String,
    pub gas_limit: String,
    pub gas_used: String,
    pub timestamp: i64,
    pub extra_data: String,
    pub mix_hash: String,
    pub nonce: String,
    pub base_fee_per_gas: Option<String>,
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
#[diesel(table_name = crate::schema::transaction_receipts)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct TransactionReceipt {
    pub chain_id: i64,
    pub transaction_hash: String,
    pub transaction_nonce: String,
    pub block_hash: String,
    pub block_number: i64,
    pub transaction_index: i64,
    pub _from: String,
    pub _to: Option<String>,
    pub cumulative_gas_used: String,
    pub gas_used: String,
    pub contract_address: Option<String>,
    pub transaction_status: String,
    pub logs_bloom: String,
    pub transaction_type: String,
    pub effective_gas_price: Option<String>,
}

impl BlockModel {
    pub fn into_alloy(self, transactions: BlockTransactions<AlloyTransaction>) -> AlloyBlock {
        AlloyBlock {
            header: Header {
                hash: self.block_hash.parse().unwrap(),
                parent_hash: self.parent_hash.parse().unwrap(),
                uncles_hash: self.ommers_hash.parse().unwrap(),
                state_root: self.state_root.parse().unwrap(),
                transactions_root: self.transactions_root.parse().unwrap(),
                receipts_root: self.receipts_root.parse().unwrap(),
                number: self.block_number as u64,
                gas_used: self.gas_used.parse::<U256>().unwrap().to(),
                gas_limit: self.gas_limit.parse::<U256>().unwrap().to(),
                extra_data: self.extra_data.parse().unwrap(),
                logs_bloom: self.logs_bloom.parse().unwrap(),
                timestamp: self.timestamp as u64,
                difficulty: self.difficulty.parse().unwrap(),
                total_difficulty: Some(Default::default()),
                mix_hash: Some(self.mix_hash.parse().unwrap()),
                nonce: Some(self.nonce.parse().unwrap()),
                base_fee_per_gas: self
                    .base_fee_per_gas
                    .map(|x| x.parse::<U256>().unwrap().to()),
                miner: self.beneficiary.parse().unwrap(),
                ..Default::default()
            },
            size: Some(Default::default()),
            transactions,
            ..Default::default()
        }
    }

    pub fn from(chain_id: u64, value: &AlloyBlock) -> Self {
        Self {
            block_number: value.header.number as i64,
            block_hash: value.header.hash.to_hex_string(),
            parent_hash: value.header.parent_hash.to_hex_string(),
            ommers_hash: value.header.uncles_hash.to_hex_string(),
            beneficiary: value.header.miner.to_hex_string(),
            state_root: value.header.state_root.to_hex_string(),
            transactions_root: value.header.transactions_root.to_hex_string(),
            receipts_root: value.header.receipts_root.to_hex_string(),
            logs_bloom: value.header.logs_bloom.0.to_hex_string(),
            difficulty: value.header.difficulty.to_hex_string(),
            gas_limit: value.header.gas_limit.to_hex_string(),
            gas_used: value.header.gas_used.to_hex_string(),
            timestamp: value.header.timestamp as i64,
            extra_data: value.header.extra_data.to_hex_string(),
            mix_hash: value.header.mix_hash.unwrap().to_hex_string(),
            nonce: value.header.nonce.unwrap().to_hex_string(),
            base_fee_per_gas: value.header.base_fee_per_gas.map(|gas| gas.to_hex_string()),
            chain_id: chain_id.try_into().unwrap(),
        }
    }
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

impl TransactionReceipt {
    pub fn from(chain_id: u64, value: &AlloyReceipt) -> Self {
        let result = Self {
            chain_id: chain_id.try_into().unwrap(),
            transaction_hash: value.transaction_hash.to_hex_string(),
            transaction_nonce: value.transaction_index.unwrap().to_hex_string(),
            block_hash: value.block_hash.unwrap_or_default().to_hex_string(),
            block_number: value.block_number.unwrap() as i64,
            transaction_index: value.transaction_index.unwrap() as i64,
            _from: value.from.to_hex_string(),
            _to: value.to.map(|x| x.to_hex_string()),
            cumulative_gas_used: value.inner.cumulative_gas_used().to_hex_string(),
            gas_used: value.gas_used.to_hex_string(),
            contract_address: value.contract_address.map(|x| x.to_hex_string()),
            transaction_status: value.inner.status().to_hex_string(),
            logs_bloom: value.inner.logs_bloom().0.to_hex_string(),
            transaction_type: u8::from(value.inner.tx_type()).to_hex_string(),
            effective_gas_price: Some(value.effective_gas_price.to_hex_string()),
        };

        result
    }

    pub fn build_envelope<T>(&self, logs: Vec<T>) -> ReceiptEnvelope<T> {
        let inner = ReceiptWithBloom::new(
            Receipt {
                status: (self.transaction_status.parse::<U256>().unwrap() != U256::ZERO).into(),
                cumulative_gas_used: self.cumulative_gas_used.parse::<U256>().unwrap().to(),
                logs,
            },
            self.logs_bloom.parse().unwrap(),
        );
        match self.transaction_type.parse::<U256>().unwrap().to::<u8>() {
            0 => ReceiptEnvelope::Legacy(inner),
            1 => ReceiptEnvelope::Eip2930(inner),
            2 => ReceiptEnvelope::Eip1559(inner),
            3 => ReceiptEnvelope::Eip4844(inner),
            _ => panic!(),
        }
    }

    pub fn into_alloy<T: Into<Log>>(self, logs: Vec<T>) -> AlloyReceipt {
        let inner = self.build_envelope(logs.into_iter().map(Into::into).collect());
        AlloyReceipt {
            transaction_hash: self.transaction_hash.parse().unwrap(),
            transaction_index: Some(self.transaction_index as u64),
            block_hash: Some(self.block_hash.parse().unwrap()),
            block_number: Some(self.block_number as u64),
            from: self._from.parse().unwrap(),
            to: self._to.map(|x| x.parse().unwrap()),
            gas_used: self.gas_used.parse::<U256>().unwrap().to(),
            contract_address: self.contract_address.map(|x| x.parse().unwrap()),
            effective_gas_price: self
                .effective_gas_price
                .map(|x| x.parse::<U256>().unwrap().to())
                .unwrap_or_default(),
            blob_gas_used: Default::default(),
            blob_gas_price: Default::default(),
            state_root: Default::default(),
            authorization_list: Default::default(),
            inner,
        }
    }
}
