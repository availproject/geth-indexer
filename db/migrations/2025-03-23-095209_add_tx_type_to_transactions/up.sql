-- Your SQL goes here

ALTER TABLE transactions ADD COLUMN tx_type TEXT NOT NULL DEFAULT 'native';
