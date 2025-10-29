-- This file should undo anything in `up.sql`
ALTER TABLE couriers
DROP COLUMN rating_sum,
  DROP COLUMN rating_count;