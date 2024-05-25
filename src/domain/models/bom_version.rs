use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::domain::BOMChangeEvent;

#[derive(Debug, Clone, PartialEq)]
pub struct BomVersion {
    pub id: Uuid,
    pub bom_id: Uuid,
    pub version: i32,
    pub changes: Box<Vec<BOMChangeEvent>>,
    pub created_at: DateTime<Utc>,
}

impl BomVersion {
    pub fn new(bom_id: &Uuid, version: i32, changes: Box<Vec<BOMChangeEvent>>) -> Self {
        Self {
            id: Uuid::new_v4(),
            bom_id: *bom_id,
            version,
            changes,
            created_at: Utc::now(),
        }
    }
}
