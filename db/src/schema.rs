// @generated automatically by Diesel CLI.

diesel::table! {
    chains (chain_id) {
        chain_id -> Int8,
        latest_tps -> Int8,
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

diesel::joinable!(transactions -> chains (chain_id));

diesel::allow_tables_to_appear_in_same_query!(chains, transactions,);
