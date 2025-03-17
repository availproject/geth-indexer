// @generated automatically by Diesel CLI.

diesel::table! {
    blocks (chain_id, block_number) {
        chain_id -> Int8,
        block_number -> Int8,
        block_hash -> Text,
        parent_hash -> Text,
        ommers_hash -> Text,
        beneficiary -> Text,
        state_root -> Text,
        transactions_root -> Text,
        receipts_root -> Text,
        logs_bloom -> Text,
        difficulty -> Text,
        gas_limit -> Text,
        gas_used -> Text,
        timestamp -> Int8,
        extra_data -> Text,
        mix_hash -> Text,
        nonce -> Text,
        base_fee_per_gas -> Nullable<Text>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    chains (chain_id) {
        chain_id -> Int8,
        latest_tps -> Int8,
    }
}

diesel::table! {
    transaction_receipts (chain_id, transaction_hash) {
        chain_id -> Int8,
        transaction_hash -> Text,
        transaction_nonce -> Text,
        block_hash -> Text,
        block_number -> Int8,
        transaction_index -> Int8,
        _from -> Text,
        _to -> Nullable<Text>,
        cumulative_gas_used -> Text,
        gas_used -> Text,
        contract_address -> Nullable<Text>,
        transaction_status -> Text,
        logs_bloom -> Text,
        transaction_type -> Text,
        effective_gas_price -> Nullable<Text>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    transactions (chain_id, transaction_hash) {
        chain_id -> Int8,
        transaction_hash -> Text,
        transaction_nonce -> Text,
        block_hash -> Nullable<Text>,
        block_number -> Nullable<Int8>,
        transaction_index -> Nullable<Int8>,
        _from -> Text,
        _to -> Nullable<Text>,
        value -> Text,
        gas_price -> Nullable<Text>,
        gas -> Text,
        input -> Text,
        v -> Text,
        r -> Text,
        s -> Text,
        transaction_type -> Text,
        impersonated -> Bool,
        max_priority_fee_per_gas -> Nullable<Text>,
        max_fee_per_gas -> Nullable<Text>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::joinable!(blocks -> chains (chain_id));
diesel::joinable!(transaction_receipts -> chains (chain_id));
diesel::joinable!(transactions -> chains (chain_id));

diesel::allow_tables_to_appear_in_same_query!(blocks, chains, transaction_receipts, transactions,);
