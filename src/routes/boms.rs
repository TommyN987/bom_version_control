use std::{
    fmt::{self, Display, Formatter},
    sync::Arc,
};

use actix_web::{get, post, put, web, HttpResponse};
use anyhow::anyhow;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    db::{
        models::{db_bom::DbBOM, db_boms_component::DbBOMComponent, db_component::DbComponent},
        operations::{
            find_bom_by_id, find_multiple_components, get_components_of_bom_by_id, insert_bom,
            update_and_archive_bom_by_id,
        },
        DbPool,
    },
    domain::{BOMChangeEvent, BOM},
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

impl Display for NewBOM {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "NewBOM {{ name: {}, description: {:?}, components: {:?} }}",
            self.name, self.description, self.components
        )
    }
}

impl TryFrom<&NewBOM> for DbBOM {
    type Error = ApiError;

    fn try_from(value: &NewBOM) -> Result<Self, Self::Error> {
        if value.name.is_empty() {
            return Err(ApiError::BadRequest("Name is required".to_string()));
        }

        if value.components.is_empty() {
            return Err(ApiError::BadRequest("Components are required".to_string()));
        }

        Ok(Self {
            id: Uuid::new_v4(),
            name: value.name.clone(),
            description: value.description.clone(),
            version: 1,
            created_at: Utc::now(),
        })
    }
}

async fn retrieve_bom(pool: Arc<DbPool>, id: Uuid) -> Result<BOM, ApiError> {
    let mut conn = pool.get().map_err(|e| anyhow!(e))?;
    let (db_bom, db_bom_components, db_components) = actix_web::web::block(
        move || -> Result<(DbBOM, Vec<DbBOMComponent>, Vec<DbComponent>), ApiError> {
            let db_bom: DbBOM = find_bom_by_id(&mut conn, id)?;
            let (db_bom_components, db_components) = get_components_of_bom_by_id(&mut conn, id)?;
            Ok((db_bom, db_bom_components, db_components))
        },
    )
    .await??;

    Ok(BOM::try_from((db_bom, db_bom_components, db_components))?)
}

#[tracing::instrument(name = "Getting BOM by ID", skip(pool, id), fields(request_id = %Uuid::new_v4()))]
#[get("/boms/{id}")]
pub async fn get_bom_by_id(
    pool: web::Data<DbPool>,
    id: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let bom: BOM = retrieve_bom(pool.into_inner(), id.into_inner()).await?;
    Ok(HttpResponse::Ok().json(bom))
}

#[tracing::instrument(name = "Creating BOM", skip(pool), fields(request_id = %Uuid::new_v4(), new_bom = %new_bom))]
#[post("/boms")]
pub async fn create_bom(
    pool: web::Data<DbPool>,
    new_bom: web::Json<NewBOM>,
) -> Result<HttpResponse, ApiError> {
    let mut conn = pool.get().map_err(|e| anyhow!(e))?;

    let new_bom = new_bom.into_inner();
    let new_db_bom: DbBOM = DbBOM::try_from(&new_bom)?;
    let new_db_bom_id = &new_db_bom.id.clone();

    let db_bom_comp_vec: Vec<DbBOMComponent> = new_bom
        .components
        .iter()
        .map(|(comp_id, qty)| DbBOMComponent::new(*new_db_bom_id, *comp_id, *qty))
        .collect();

    println!("New BOMComponents: {:?}", db_bom_comp_vec);

    let (db_bom, db_bom_components, db_components) = actix_web::web::block(
        move || -> Result<(DbBOM, Vec<DbBOMComponent>, Vec<DbComponent>), ApiError> {
            let (db_bom, db_bom_components) =
                insert_bom(&mut conn, new_db_bom, db_bom_comp_vec.to_vec())?;
            let db_components = find_multiple_components(
                &mut conn,
                new_bom
                    .components
                    .iter()
                    .map(|(id, _)| *id)
                    .collect::<Vec<Uuid>>(),
            )?;
            Ok((db_bom, db_bom_components, db_components))
        },
    )
    .await??;

    Ok(HttpResponse::Created().json(BOM::try_from((db_bom, db_bom_components, db_components))?))
}

#[tracing::instrument(name = "Updating BOM", skip(pool), fields(request_id = %Uuid::new_v4(), id = %id))]
#[put("/boms/{id}")]
pub async fn update_bom(
    pool: web::Data<DbPool>,
    id: web::Path<Uuid>,
    change_events: web::Json<Vec<BOMChangeEvent>>,
) -> Result<HttpResponse, ApiError> {
    let change_events = change_events.into_inner();
    let bom_id = id.into_inner();

    let mut conn = pool.get().map_err(|e| anyhow!(e))?;

    let (updated_bom, updated_bom_components, updated_components) = actix_web::web::block(
        move || -> Result<(DbBOM, Vec<DbBOMComponent>, Vec<DbComponent>), anyhow::Error> {
            update_and_archive_bom_by_id(&mut conn, bom_id, change_events)
        },
    )
    .await??;

    Ok(HttpResponse::Created().json(BOM::try_from((
        updated_bom,
        updated_bom_components,
        updated_components,
    ))?))
}
