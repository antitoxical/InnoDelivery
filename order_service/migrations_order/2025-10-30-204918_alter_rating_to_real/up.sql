-- Your SQL goes here
ALTER TABLE orders ALTER COLUMN rating TYPE REAL USING rating::REAL;