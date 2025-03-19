-- This file should undo anything in `up.sql`

DROP INDEX IF EXISTS idx_transactions_hash;
DROP INDEX IF EXISTS idx_transactions_block_hash;
DROP INDEX IF EXISTS idx_transactions_block_number;
DROP INDEX IF EXISTS idx_transactions_chain_id;
DROP INDEX IF EXISTS idx_transactions_index;
DROP INDEX IF EXISTS idx_transactions_from;
DROP INDEX IF EXISTS idx_transactions_to;
DROP INDEX IF EXISTS idx_transactions_type;
DROP INDEX IF EXISTS idx_transactions_impersonated;
