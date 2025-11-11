-- Your SQL goes here
ALTER TABLE order_products DROP CONSTRAINT order_products_pkey;
ALTER TABLE order_products DROP COLUMN id;
ALTER TABLE order_products ADD PRIMARY KEY (order_id, product_id);