use chrono::{DateTime, Utc};
use diesel::{
    prelude::{Insertable, Queryable},
    Selectable,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{domain::BOM, schema::boms};

use super::db_component::DbComponent;

#[derive(Debug, PartialEq, Deserialize, Insertable, Queryable, Selectable)]
#[diesel(table_name = boms)]
pub struct DbBOM {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub version: i32,
    pub created_at: DateTime<Utc>,
}

impl From<(DbBOM, Vec<DbComponent>, Vec<i32>)> for BOM {
    fn from(value: (DbBOM, Vec<DbComponent>, Vec<i32>)) -> Self {
        Self {
            id: value.0.id,
            name: value.0.name,
            description: value.0.description,
            version: value.0.version,
            components: value.1.into_iter().map(|c| c.into()).zip(value.2).collect(),
            created_at: value.0.created_at,
        }
    }
}
