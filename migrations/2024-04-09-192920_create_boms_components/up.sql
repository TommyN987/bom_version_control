-- Your SQL goes here

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE boms_components (
    bom_id UUID NOT NULL,
    component_id UUID NOT NULL,
    quantity INTEGER NOT NULL,
    FOREIGN KEY (bom_id) REFERENCES boms(id) ON DELETE CASCADE,
    FOREIGN KEY (component_id) REFERENCES components(id) ON DELETE CASCADE,
    PRIMARY KEY(bom_id, component_id)
);
