use actix_web::{get, post, put, web, HttpResponse};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    domain::{newtypes::new_bom::NewBOM, BOMChangeEvent, BOM},
    services::bom_service::{BomService, UpdateOperation},
};

use super::ApiError;

#[tracing::instrument(name = "Getting BOMs", skip(bom_service), fields(request_id = %Uuid::new_v4()))]
#[get("/boms")]
pub async fn get_all_boms(bom_service: web::Data<BomService>) -> Result<HttpResponse, ApiError> {
    let boms: Vec<BOM> = actix_web::web::block(move || bom_service.find_all_boms()).await??;
    Ok(HttpResponse::Ok().json(boms))
}

#[tracing::instrument(name = "Getting BOM by ID", skip(bom_service, id), fields(request_id = %Uuid::new_v4()))]
#[get("/boms/{id}")]
pub async fn get_bom_by_id(
    bom_service: web::Data<BomService>,
    id: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let bom: BOM =
        actix_web::web::block(move || bom_service.find_bom_by_id(id.into_inner())).await??;
    Ok(HttpResponse::Ok().json(bom))
}

#[tracing::instrument(name = "Creating BOM", skip(bom_service), fields(request_id = %Uuid::new_v4(), new_bom = %new_bom))]
#[post("/boms")]
pub async fn create_bom(
    bom_service: web::Data<BomService>,
    new_bom: web::Json<NewBOM>,
) -> Result<HttpResponse, ApiError> {
    let new_bom = new_bom.into_inner();
    let new_bom: BOM = actix_web::web::block(move || bom_service.insert_bom(new_bom)).await??;

    Ok(HttpResponse::Created().json(new_bom))
}

#[tracing::instrument(name = "Updating BOM", skip(bom_service), fields(request_id = %Uuid::new_v4(), id = %id))]
#[put("/boms/{id}")]
pub async fn update_bom(
    bom_service: web::Data<BomService>,
    id: web::Path<Uuid>,
    change_events: web::Json<Vec<BOMChangeEvent>>,
) -> Result<HttpResponse, ApiError> {
    let change_events = change_events.into_inner();
    let bom_id = id.into_inner();

    let updated_bom: BOM = actix_web::web::block(move || {
        bom_service.update_bom(bom_id, change_events, UpdateOperation::Incremental)
    })
    .await??;

    Ok(HttpResponse::Created().json(updated_bom))
}

#[derive(Deserialize)]
pub struct VersionRange {
    pub from: i32,
    pub to: i32,
}

#[get("/boms/{id}/diffs")]
pub async fn get_bom_diff(
    bom_service: web::Data<BomService>,
    id: web::Path<Uuid>,
    params: web::Query<VersionRange>,
) -> Result<HttpResponse, ApiError> {
    let bom_id = id.into_inner();
    let params = params.into_inner();

    let diff =
        actix_web::web::block(move || bom_service.get_bom_diff(bom_id, params.from, params.to))
            .await??;

    Ok(HttpResponse::Ok().json(diff))
}

#[derive(Deserialize)]
pub struct VersionQuery {
    version: i32,
}

#[get("/boms/{id}/")]
pub async fn get_bom_version(
    bom_service: web::Data<BomService>,
    id: web::Path<Uuid>,
    version: web::Query<VersionQuery>,
) -> Result<HttpResponse, ApiError> {
    let bom_id = id.into_inner();
    let version = version.into_inner();

    let bom = actix_web::web::block(move || {
        bom_service.find_bom_by_version_and_id(bom_id, version.version)
    })
    .await??;

    Ok(HttpResponse::Ok().json(bom))
}

#[derive(Deserialize)]
struct RevertBOM {
    revert_to_version: i32,
}

#[put("/boms/{id}/")]
pub async fn revert_bom_to_version(
    bom_service: web::Data<BomService>,
    id: web::Path<Uuid>,
    version: web::Query<RevertBOM>,
) -> Result<HttpResponse, ApiError> {
    let bom_id = id.into_inner();
    let version = version.into_inner();

    let reverted_bom = actix_web::web::block(move || {
        bom_service.revert_bom_to_version(bom_id, version.revert_to_version)
    })
    .await??;

    Ok(HttpResponse::Created().json(reverted_bom))
}
