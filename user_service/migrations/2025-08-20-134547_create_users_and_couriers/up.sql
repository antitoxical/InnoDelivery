-- Your SQL goes here
-- up.sql

CREATE TABLE users (
                       id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                       name VARCHAR(255) NOT NULL,
                       phone_number VARCHAR(50) NOT NULL UNIQUE,
                       email VARCHAR(255) NOT NULL UNIQUE,
                       password VARCHAR(255) NOT NULL,
                       role VARCHAR(50) NOT NULL,
                       favorite_address VARCHAR(255),
                       is_blocked BOOLEAN NOT NULL DEFAULT FALSE,
                       is_deleted BOOLEAN NOT NULL DEFAULT FALSE,
                       created_at TIMESTAMP NOT NULL DEFAULT NOW(),
                       updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE TABLE couriers (
                          id UUID PRIMARY KEY,
                          user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
                          status VARCHAR(50) NOT NULL DEFAULT 'free',
                          rating REAL NOT NULL DEFAULT 0.0,
                          created_at TIMESTAMP NOT NULL DEFAULT NOW(),
                          updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- Automatically update updated_at
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_users_updated_at BEFORE UPDATE ON users FOR EACH ROW EXECUTE PROCEDURE update_updated_at_column();
CREATE TRIGGER update_couriers_updated_at BEFORE UPDATE ON couriers FOR EACH ROW EXECUTE PROCEDURE update_updated_at_column();