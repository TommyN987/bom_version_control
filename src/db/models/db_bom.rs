use std::collections::HashMap;

use anyhow::anyhow;
use chrono::{DateTime, Utc};
use diesel::{
    prelude::{Insertable, Queryable},
    Selectable,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    domain::{Component, BOM},
    schema::boms,
};

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

impl TryFrom<(DbBOM, HashMap<Uuid, (Option<Component>, i32)>)> for BOM {
    type Error = anyhow::Error;

    fn try_from(
        value: (DbBOM, HashMap<Uuid, (Option<Component>, i32)>),
    ) -> Result<Self, Self::Error> {
        let (db_bom, comp_map) = value;

        let components = comp_map
            .into_iter()
            .map(|(_, (comp, qty))| {
                comp.ok_or_else(|| anyhow!("Component not found"))
                    .map(|c| (c, qty))
            })
            .collect::<Result<Vec<(Component, i32)>, _>>()?;

        Ok(Self {
            id: db_bom.id,
            name: db_bom.name,
            description: db_bom.description,
            version: db_bom.version,
            components,
            created_at: db_bom.created_at,
        })
    }
}
