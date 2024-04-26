use std::collections::HashMap;

use anyhow::anyhow;
use chrono::{DateTime, Utc};
use diesel::{
    associations::Identifiable,
    prelude::{Insertable, Queryable},
    query_builder::AsChangeset,
    Selectable,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    domain::{Component, BOM},
    schema::boms,
};

use super::{db_boms_component::DbBOMComponent, db_component::DbComponent};

#[derive(
    Debug, PartialEq, Deserialize, Identifiable, Insertable, Queryable, Selectable, AsChangeset,
)]
#[diesel(table_name = boms)]
pub struct DbBOM {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub version: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<&BOM> for DbBOM {
    fn from(value: &BOM) -> Self {
        Self {
            id: value.id,
            name: value.name.clone(),
            description: value.description.clone(),
            version: value.version,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

impl TryFrom<(DbBOM, Vec<DbBOMComponent>, Vec<DbComponent>)> for BOM {
    type Error = anyhow::Error;

    fn try_from(
        value: (DbBOM, Vec<DbBOMComponent>, Vec<DbComponent>),
    ) -> Result<Self, Self::Error> {
        let (db_bom, db_bom_components, db_components) = value;

        let mut comp_map: HashMap<Uuid, (Option<Component>, i32)> = HashMap::new();

        db_bom_components.iter().for_each(|b_c| {
            comp_map.insert(b_c.component_id, (None, b_c.quantity));
        });

        db_components.into_iter().for_each(|c| {
            comp_map
                .entry(c.id)
                .and_modify(|(comp, _)| *comp = Some(c.into()));
        });

        Ok(Self {
            id: db_bom.id,
            name: db_bom.name,
            description: db_bom.description,
            version: db_bom.version,
            components: comp_map
                .into_values()
                .map(|(comp, qty)| comp.ok_or(anyhow!("Component not found")).map(|c| (c, qty)))
                .collect::<Result<Vec<(Component, i32)>, _>>()?,
            created_at: db_bom.created_at,
            updated_at: db_bom.updated_at,
        })
    }
}
