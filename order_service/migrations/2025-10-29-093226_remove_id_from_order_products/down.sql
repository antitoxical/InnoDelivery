-- This file should undo anything in `up.sql`
ALTER TABLE order_products DROP CONSTRAINT order_products_pkey;
ALTER TABLE order_products ADD COLUMN id UUID PRIMARY KEY DEFAULT uuid_generate_v4();
