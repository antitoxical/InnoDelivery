-- This file should undo anything in `up.sql`
ALTER TABLE orders ALTER COLUMN rating TYPE INTEGER USING rating::INTEGER;
