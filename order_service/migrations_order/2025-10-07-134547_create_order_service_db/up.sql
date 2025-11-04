CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TYPE order_status AS ENUM (
    'draft',
    'pending_carrier',
    'in_progress',
    'finished',
    'cancelled'
);

CREATE TABLE products (
                          id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
                          product_type VARCHAR NOT NULL,
                          product_name VARCHAR NOT NULL,
                          restaurant VARCHAR NOT NULL,
                          price REAL NOT NULL,
                          created_at TIMESTAMP NOT NULL DEFAULT NOW(),
                          updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE TABLE orders (
                        id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
                        user_id UUID NOT NULL,
                        courier_id UUID,
                        delivery_address VARCHAR NOT NULL,
                        status order_status NOT NULL DEFAULT 'draft',
                        rating REAL NOT NULL DEFAULT 0.0,
                        created_at TIMESTAMP NOT NULL DEFAULT NOW(),
                        updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
                        CONSTRAINT rating_check CHECK (rating IS NULL OR (rating >= 1 AND rating <= 5))
);

CREATE TABLE order_products (
                                id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
                                order_id UUID NOT NULL REFERENCES orders(id) ON DELETE CASCADE,
                                product_id UUID NOT NULL REFERENCES products(id),
                                quantity INT NOT NULL DEFAULT 1
);
