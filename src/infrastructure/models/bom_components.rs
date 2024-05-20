use diesel::{Associations, Identifiable, Insertable, Queryable, Selectable};
use uuid::Uuid;

use super::{bom::BOM as Bom, component::Component};

use crate::schema::boms_components;

#[derive(
    Debug, Clone, PartialEq, Insertable, Queryable, Selectable, Identifiable, Associations,
)]
#[diesel(belongs_to(Bom))]
#[diesel(belongs_to(Component))]
#[diesel(table_name = boms_components)]
#[diesel(primary_key(bom_id, component_id))]
pub struct BomComponent {
    pub bom_id: Uuid,
    pub component_id: Uuid,
    pub quantity: i32,
}

impl BomComponent {
    pub fn new(bom_id: Uuid, component_id: Uuid, quantity: i32) -> Self {
        Self {
            bom_id,
            component_id,
            quantity,
        }
    }
}
