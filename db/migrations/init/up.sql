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
    tx_type TEXT,

    PRIMARY KEY (chain_id, transaction_hash)
);

ALTER TABLE chains 
ADD COLUMN created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL,
ADD COLUMN updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL;

SELECT diesel_manage_updated_at('chains');

ALTER TABLE transactions 
ADD COLUMN created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL,
ADD COLUMN updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL;

SELECT diesel_manage_updated_at('transactions');

CREATE INDEX IF NOT EXISTS idx_transactions_hash ON transactions (transaction_hash);
CREATE INDEX IF NOT EXISTS idx_transactions_block_hash ON transactions (block_hash);
CREATE INDEX IF NOT EXISTS idx_transactions_block_number ON transactions (block_number);
CREATE INDEX IF NOT EXISTS idx_transactions_chain_id ON transactions (chain_id);
CREATE INDEX IF NOT EXISTS idx_transactions_index ON transactions (chain_id, block_number, transaction_index);
CREATE INDEX IF NOT EXISTS idx_transactions_from ON transactions (_from);
CREATE INDEX IF NOT EXISTS idx_transactions_to ON transactions (_to);
CREATE INDEX IF NOT EXISTS idx_transactions_type ON transactions (transaction_type);
CREATE INDEX IF NOT EXISTS idx_transactions_impersonated ON transactions (impersonated);

