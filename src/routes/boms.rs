use actix_web::{post, web, HttpResponse};
use anyhow::anyhow;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    db::{models::db_bom::DbBOM, operations::insert_bom, DbPool},
    domain::BOM,
};

use super::ApiError;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct NewBOM {
    name: String,
    description: Option<String>,
    components: Vec<(Uuid, i32)>,
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

#[post("/boms")]
pub async fn create_bom(
    pool: web::Data<DbPool>,
    new_bom: web::Json<NewBOM>,
) -> Result<HttpResponse, ApiError> {
    let mut conn = pool.get().map_err(|e| anyhow!(e))?;

    let new_bom = new_bom.into_inner();
    let new_db_bom: DbBOM = DbBOM::try_from(&new_bom)?;

    let bom: BOM =
        actix_web::web::block(move || insert_bom(&mut conn, new_db_bom, new_bom.components))
            .await??;

    Ok(HttpResponse::Created().json(bom))
}
