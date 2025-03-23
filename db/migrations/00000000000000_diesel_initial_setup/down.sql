DROP FUNCTION IF EXISTS diesel_manage_updated_at(_tbl regclass);
DROP FUNCTION IF EXISTS diesel_set_updated_at();

DROP TABLE transaction_receipts;
DROP TABLE transactions;
DROP TABLE blocks;
DROP TABLE chains;

ALTER TABLE blocks DROP COLUMN IF EXISTS created_at, DROP COLUMN IF EXISTS updated_at;
DROP TRIGGER IF EXISTS set_updated_at ON blocks;

ALTER TABLE chains DROP COLUMN IF EXISTS created_at, DROP COLUMN IF EXISTS updated_at;
DROP TRIGGER IF EXISTS set_updated_at ON chains;

ALTER TABLE transaction_receipts DROP COLUMN IF EXISTS created_at, DROP COLUMN IF EXISTS updated_at;
DROP TRIGGER IF EXISTS set_updated_at ON transaction_receipts;

ALTER TABLE transactions DROP COLUMN IF EXISTS created_at, DROP COLUMN IF EXISTS updated_at;
DROP TRIGGER IF EXISTS set_updated_at ON transactions;

DROP INDEX IF EXISTS idx_transactions_hash;
DROP INDEX IF EXISTS idx_transactions_block_hash;
DROP INDEX IF EXISTS idx_transactions_block_number;
DROP INDEX IF EXISTS idx_transactions_chain_id;
DROP INDEX IF EXISTS idx_transactions_index;
DROP INDEX IF EXISTS idx_transactions_from;
DROP INDEX IF EXISTS idx_transactions_to;
DROP INDEX IF EXISTS idx_transactions_type;
DROP INDEX IF EXISTS idx_transactions_impersonated;
