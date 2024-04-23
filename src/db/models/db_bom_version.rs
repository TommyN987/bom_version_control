use chrono::{DateTime, Utc};
use diesel::{prelude::Insertable, Associations, Identifiable, Queryable, Selectable};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use super::db_bom::DbBOM as Bom;

use crate::{domain::BOMChangeEvent, schema::bom_versions};

#[derive(
    Debug,
    PartialEq,
    Serialize,
    Deserialize,
    Identifiable,
    Selectable,
    Queryable,
    Insertable,
    Associations,
)]
#[diesel(belongs_to(Bom))]
#[diesel(table_name = bom_versions)]
pub struct BOMVersion {
    pub id: Uuid,
    pub bom_id: Uuid,
    pub version: i32,
    pub changes: Value,
    pub created_at: DateTime<Utc>,
}

impl From<(&Bom, &Vec<BOMChangeEvent>)> for BOMVersion {
    fn from(value: (&Bom, &Vec<BOMChangeEvent>)) -> Self {
        Self {
            id: Uuid::new_v4(),
            bom_id: value.0.id,
            version: value.0.version,
            changes: serde_json::json!(value.1),
            created_at: Utc::now(),
        }
    }
}
