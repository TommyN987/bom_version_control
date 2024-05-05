-- Your SQL goes here

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE components (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR NOT NULL,
    part_number VARCHAR NOT NULL,
    description TEXT,
    supplier VARCHAR NOT NULL,
    price_value REAL NOT NULL,
    price_currency VARCHAR NOT NULL
);

CREATE INDEX components_full_text_idx
ON components
USING GIN(
    to_tsvector('english', coalesce(name, '') || ' ' || coalesce(part_number, '') || ' ' || coalesce(description, '') || ' ' || coalesce(supplier, ''))
);