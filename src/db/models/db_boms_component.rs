use diesel::{
    associations::{Associations, Identifiable},
    deserialize::Queryable,
    prelude::Insertable,
    Selectable,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::schema::boms_components;

use super::{db_bom::DbBOM as Bom, db_component::DbComponent as Component};

#[derive(
    Debug,
    PartialEq,
    Serialize,
    Deserialize,
    Insertable,
    Queryable,
    Selectable,
    Identifiable,
    Associations,
)]
#[diesel(belongs_to(Bom))]
#[diesel(belongs_to(Component))]
#[diesel(table_name = boms_components)]
#[diesel(primary_key(bom_id, component_id))]
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
