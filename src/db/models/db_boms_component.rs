use diesel::{deserialize::Queryable, prelude::Insertable};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::schema::boms_components;

#[derive(Debug, PartialEq, Serialize, Deserialize, Insertable, Queryable)]
#[diesel(table_name = boms_components)]
pub struct DbBOMComponent {
    pub bom_id: Uuid,
    pub component_id: Uuid,
    pub quantity: i32,
}

impl DbBOMComponent {
    pub fn new(bom_id: Uuid, component_id: Uuid, quantity: i32) -> Self {
        Self {
            bom_id,
            component_id,
            quantity,
        }
    }
}
