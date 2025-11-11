-- This file should undo anything in `up.sql`
ALTER TABLE orders ALTER COLUMN rating SET NOT NULL;
ALTER TABLE orders ALTER COLUMN rating SET DEFAULT 0.0;