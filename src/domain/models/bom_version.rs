use uuid::Uuid;

use crate::domain::BOMChangeEvent;

#[derive(Debug, Clone, PartialEq)]
pub struct BomVersion {
    pub id: Uuid,
    pub bom_id: Uuid,
    pub version: i32,
    pub changes: Vec<BOMChangeEvent>,
}

impl BomVersion {
    pub fn new(bom_id: &Uuid, version: i32, changes: Vec<BOMChangeEvent>) -> Self {
        Self {
            id: Uuid::new_v4(),
            bom_id: bom_id.clone(),
            version,
            changes,
        }
    }
}
