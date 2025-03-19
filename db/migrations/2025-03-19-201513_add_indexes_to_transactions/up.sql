-- Your SQL goes here

CREATE INDEX IF NOT EXISTS idx_transactions_hash ON transactions (transaction_hash);
CREATE INDEX IF NOT EXISTS idx_transactions_block_hash ON transactions (block_hash);
CREATE INDEX IF NOT EXISTS idx_transactions_block_number ON transactions (block_number);
CREATE INDEX IF NOT EXISTS idx_transactions_chain_id ON transactions (chain_id);
CREATE INDEX IF NOT EXISTS idx_transactions_index ON transactions (chain_id, block_number, transaction_index);
CREATE INDEX IF NOT EXISTS idx_transactions_from ON transactions (_from);
CREATE INDEX IF NOT EXISTS idx_transactions_to ON transactions (_to);
CREATE INDEX IF NOT EXISTS idx_transactions_type ON transactions (transaction_type);
CREATE INDEX IF NOT EXISTS idx_transactions_impersonated ON transactions (impersonated);
