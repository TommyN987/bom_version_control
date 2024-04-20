use std::collections::HashMap;

use actix_web::{post, web, HttpResponse};
use anyhow::anyhow;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    db::{
        models::{db_bom::DbBOM, db_boms_component::DbBOMComponent},
        operations::insert_bom,
        DbPool,
    },
    domain::{Component, BOM},
};

use super::ApiError;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct NewBOM {
    name: String,
    description: Option<String>,
    components: Vec<(Uuid, i32)>,
}

impl NewBOM {
    pub fn new(name: String, description: Option<String>, components: Vec<(Uuid, i32)>) -> Self {
        Self {
            name,
            description,
            components,
        }
    }
}

impl TryFrom<&NewBOM> for DbBOM {
    type Error = ApiError;

    fn try_from(value: &NewBOM) -> Result<Self, Self::Error> {
        Ok(Self {
            id: Uuid::new_v4(),
            name: value.name.clone(),
            description: value.description.clone(),
            version: 1,
            created_at: Utc::now(),
        })
    }
}

// #[get("/boms/{id}")]
// pub async fn get_bom_by_id(
//     pool: web::Data<DbPool>,
//     id: web::Path<Uuid>,
// ) -> Result<HttpResponse, ApiError> {
//     let mut conn = pool.get().map_err(|e| anyhow!(e))?;

//     let bom = actix_web::web::block(move || crate::db::operations::find_bom_by_id(&mut conn, *id))
//         .await??;

//     Ok(HttpResponse::Ok().json(bom))
// }

#[post("/boms")]
pub async fn create_bom(
    pool: web::Data<DbPool>,
    new_bom: web::Json<NewBOM>,
) -> Result<HttpResponse, ApiError> {
    let mut conn = pool.get().map_err(|e| anyhow!(e))?;

    let new_bom = new_bom.into_inner();
    let new_db_bom: DbBOM = DbBOM::try_from(&new_bom)?;

    let mut comp_map: HashMap<Uuid, (Option<Component>, i32)> = HashMap::new();

    let mut db_bom_comp_vec: Vec<DbBOMComponent> = Vec::new();

    new_bom.components.iter().for_each(|(id, quantity)| {
        comp_map.insert(*id, (None, *quantity));
        db_bom_comp_vec.push(DbBOMComponent::new(new_db_bom.id, *id, *quantity));
    });

    let (db_bom, db_components) =
        actix_web::web::block(move || insert_bom(&mut conn, new_db_bom, db_bom_comp_vec)).await??;

    db_components.into_iter().for_each(|c| {
        comp_map
            .entry(c.id)
            .and_modify(|(comp, _)| *comp = Some(c.into()));
    });

    Ok(HttpResponse::Created().json(BOM::try_from((db_bom, comp_map))?))
}
