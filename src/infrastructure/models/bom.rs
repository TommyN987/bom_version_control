use chrono::{DateTime, Utc};
use diesel::{AsChangeset, Identifiable, Insertable, Queryable, Selectable};
use uuid::Uuid;

use crate::schema::boms;

#[derive(Debug, Clone, PartialEq, Identifiable, Insertable, Queryable, Selectable, AsChangeset)]
#[diesel(table_name = boms)]
pub struct BOM {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub version: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
