CREATE OR REPLACE FUNCTION diesel_manage_updated_at(_tbl regclass) RETURNS VOID AS $$
BEGIN
    EXECUTE format('CREATE TRIGGER set_updated_at BEFORE UPDATE ON %s
                    FOR EACH ROW EXECUTE PROCEDURE diesel_set_updated_at()', _tbl);
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION diesel_set_updated_at() RETURNS trigger AS $$
BEGIN
    IF (
        NEW IS DISTINCT FROM OLD AND
        NEW.updated_at IS NOT DISTINCT FROM OLD.updated_at
    ) THEN
        NEW.updated_at := current_timestamp;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TABLE chains (
    chain_id BIGINT NOT NULL,
    latest_tps BIGINT,
    PRIMARY KEY (chain_id)
);

CREATE TABLE blocks (
    chain_id BIGINT NOT NULL REFERENCES chains(chain_id),
    block_number BIGINT NOT NULL,
    block_hash TEXT NOT NULL,
    parent_hash TEXT NOT NULL,
    ommers_hash TEXT NOT NULL,
    beneficiary TEXT NOT NULL,
    state_root TEXT NOT NULL,
    transactions_root TEXT NOT NULL,
    receipts_root TEXT NOT NULL,
    logs_bloom TEXT NOT NULL,
    difficulty TEXT NOT NULL,
    gas_limit TEXT NOT NULL,
    gas_used TEXT NOT NULL,
    timestamp BIGINT NOT NULL,
    extra_data TEXT NOT NULL,
    mix_hash TEXT NOT NULL,
    nonce TEXT NOT NULL,
    base_fee_per_gas TEXT,

    UNIQUE (chain_id, block_hash),
    PRIMARY KEY (chain_id, block_number)
);

CREATE TABLE transactions (
    chain_id BIGINT NOT NULL REFERENCES chains(chain_id),
    transaction_hash TEXT NOT NULL,
    transaction_nonce TEXT NOT NULL,
    block_hash TEXT,
    block_number BIGINT,
    transaction_index BIGINT,
    _from TEXT NOT NULL,
    _to TEXT,
    value TEXT NOT NULL,
    gas_price TEXT,
    gas TEXT NOT NULL,
    input TEXT NOT NULL,
    v TEXT NOT NULL,
    r TEXT NOT NULL,
    s TEXT NOT NULL,
    transaction_type TEXT NOT NULL,
    impersonated BOOLEAN NOT NULL,
    max_priority_fee_per_gas TEXT,
    max_fee_per_gas TEXT,

    PRIMARY KEY (chain_id, transaction_hash)
);

CREATE TABLE transaction_receipts (
    chain_id BIGINT NOT NULL REFERENCES chains(chain_id),
    transaction_hash TEXT NOT NULL,
    transaction_nonce TEXT NOT NULL,
    block_hash TEXT NOT NULL,
    block_number BIGINT NOT NULL,
    transaction_index BIGINT NOT NULL,
    _from TEXT NOT NULL,
    _to TEXT,
    cumulative_gas_used TEXT NOT NULL,
    gas_used TEXT NOT NULL,
    contract_address TEXT,
    transaction_status TEXT NOT NULL,
    logs_bloom TEXT NOT NULL,
    transaction_type TEXT NOT NULL,
    effective_gas_price TEXT,

    PRIMARY KEY (chain_id, transaction_hash)
);

ALTER TABLE blocks 
ADD COLUMN created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL,
ADD COLUMN updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL;

SELECT diesel_manage_updated_at('blocks');

ALTER TABLE chains 
ADD COLUMN created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL,
ADD COLUMN updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL;

SELECT diesel_manage_updated_at('chains');

ALTER TABLE transaction_receipts 
ADD COLUMN created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL,
ADD COLUMN updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL;

SELECT diesel_manage_updated_at('transaction_receipts');

ALTER TABLE transactions 
ADD COLUMN created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL,
ADD COLUMN updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL;

SELECT diesel_manage_updated_at('transactions');

