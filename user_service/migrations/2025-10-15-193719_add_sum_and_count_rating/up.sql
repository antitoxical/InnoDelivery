-- Your SQL goes here
ALTER TABLE couriers
    ADD COLUMN rating_sum FLOAT NOT NULL DEFAULT 0.0,
    ADD COLUMN rating_count INTEGER NOT NULL DEFAULT 0;