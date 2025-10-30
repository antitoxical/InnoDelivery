-- This file should undo anything in `up.sql`

ALTER TABLE orders ALTER COLUMN rating DROP NOT NULL;