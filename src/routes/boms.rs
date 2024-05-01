use std::{
    fmt::{self, Display, Formatter},
    sync::Arc,
};

use actix_web::{get, post, put, web, HttpResponse};
use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::{
    db::{
        models::{db_bom::DbBOM, db_boms_component::DbBOMComponent, db_component::DbComponent},
        operations::{
            fetch_change_events_until_version, find_bom_by_id, find_boms, find_multiple_components,
            get_components_of_bom_by_id, insert_bom, update_and_archive_bom_by_id,
        },
        DbPool,
    },
    domain::{BOMChangeEvent, BOMDiff, BOM},
};

use super::ApiError;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct NewBOM {
    pub events: Vec<BOMChangeEvent>,
}

impl Display for NewBOM {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.events)
    }
}

impl TryFrom<&NewBOM> for DbBOM {
    type Error = ApiError;

    fn try_from(value: &NewBOM) -> Result<Self, Self::Error> {
        let bom = BOM::try_from(&value.events)?;
        if bom.name.is_empty() {
            return Err(ApiError::BadRequest("Name is required".to_string()));
        }

        if bom.components.is_empty() {
            return Err(ApiError::BadRequest("Components are required".to_string()));
        }

        Ok(Self {
            id: Uuid::new_v4(),
            name: bom.name.clone(),
            description: bom.description.clone(),
            version: 1,
            created_at: bom.created_at,
            updated_at: bom.updated_at,
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

#[tracing::instrument(name = "Getting BOMs", skip(pool), fields(request_id = %Uuid::new_v4()))]
#[get("/boms")]
pub async fn get_all_boms(pool: web::Data<DbPool>) -> Result<HttpResponse, ApiError> {
    let mut conn = pool.get().map_err(|e| anyhow!(e))?;
    let boms: Vec<DbBOM> = actix_web::web::block(move || find_boms(&mut conn)).await??;
    Ok(HttpResponse::Ok().json(boms))
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

    let db_bom_comp_vec: Vec<DbBOMComponent> = new_bom
        .events
        .iter()
        .filter_map(|event| {
            if let BOMChangeEvent::ComponentAdded(component, qty) = event {
                Some(DbBOMComponent::new(new_db_bom.id, component.id, *qty))
            } else {
                None
            }
        })
        .collect();

    let (db_bom, db_bom_components, db_components) = actix_web::web::block(
        move || -> Result<(DbBOM, Vec<DbBOMComponent>, Vec<DbComponent>), ApiError> {
            let (db_bom, db_bom_components) = insert_bom(
                &mut conn,
                new_db_bom,
                db_bom_comp_vec.to_vec(),
                new_bom.events,
            )?;
            let db_components = find_multiple_components(
                &mut conn,
                &db_bom_components
                    .iter()
                    .map(|db_bom_comp| db_bom_comp.component_id)
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

#[derive(Deserialize)]
pub struct VersionRange {
    pub from: i32,
    pub to: i32,
}

#[get("/boms/{id}/diffs")]
pub async fn get_bom_diff(
    pool: web::Data<DbPool>,
    id: web::Path<Uuid>,
    params: web::Query<VersionRange>,
) -> Result<HttpResponse, ApiError> {
    if params.from >= params.to {
        return Err(ApiError::BadRequest("Invalid version range".to_string()));
    }

    let mut conn = pool.get().map_err(|e| anyhow!(e))?;
    let bom_id = id.into_inner();
    let params = params.into_inner();
    let resp = actix_web::web::block(move || {
        fetch_change_events_until_version(&mut conn, bom_id, params.to - 1)
    })
    .await??;

    let mut events_until_starting_bom: Vec<BOMChangeEvent> = Vec::new();
    let mut events_until_ending_bom: Vec<BOMChangeEvent> = Vec::new();

    for (i, (_, values)) in resp.iter().enumerate() {
        if let Value::Array(array) = values {
            if i <= (params.from - 1) as usize {
                for item in array {
                    events_until_starting_bom.push(serde_json::from_value(item.clone())?);
                }
            } else {
                for item in array {
                    events_until_ending_bom.push(serde_json::from_value(item.clone())?);
                }
            }
        }
    }

    let initial_bom = BOM::try_from(&events_until_starting_bom)?;

    let diff = BOMDiff::from((&initial_bom, &events_until_ending_bom));

    Ok(HttpResponse::Ok().json(diff))
}
