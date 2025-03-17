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