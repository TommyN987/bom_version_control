use chrono::{DateTime, Utc};
use diesel::{prelude::Insertable, Associations, Identifiable, Queryable, Selectable};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use super::bom::BOM as Bom;

use crate::schema::bom_versions;

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
