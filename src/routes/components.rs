use actix_web::{get, post, web, HttpResponse};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    domain::{newtypes::new_component::NewComponent, Component},
    services::bom_service::BomService,
};

use super::ApiError;

#[tracing::instrument(name = "Getting all components", skip(bom_service), fields(request_id = %Uuid::new_v4()))]
#[get("/components")]
pub async fn get_components(bom_service: web::Data<BomService>) -> Result<HttpResponse, ApiError> {
    let components: Vec<Component> =
        actix_web::web::block(move || bom_service.find_all_components()).await??;

    Ok(HttpResponse::Ok().json(components))
}

#[tracing::instrument(name = "Getting a component by id", skip(bom_service), fields(request_id = %Uuid::new_v4(), id = %id))]
#[get("/components/{id}")]
pub async fn get_component_by_id(
    bom_service: web::Data<BomService>,
    id: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let component: Component =
        actix_web::web::block(move || bom_service.find_component_by_id(id.into_inner())).await??;

    Ok(HttpResponse::Ok().json(component))
}

#[derive(Debug, Deserialize)]
struct SearchQuery {
    q: String,
}

#[tracing::instrument(name = "Searching for components", skip(bom_service), fields(request_id = %Uuid::new_v4(), query = %query.q))]
#[get("/components/search")]
pub async fn search_components(
    bom_service: web::Data<BomService>,
    query: web::Query<SearchQuery>,
) -> Result<HttpResponse, ApiError> {
    let query = query.into_inner();
    let components: Vec<Component> =
        actix_web::web::block(move || bom_service.search_components(&query.q)).await??;

    Ok(HttpResponse::Ok().json(components))
}

#[tracing::instrument(name = "Creating a component", skip(bom_service), fields(request_id = %Uuid::new_v4(), name = %component.name, part_number = %component.part_number, supplier = %component.supplier, price = %component.price.value, currency = %component.price.currency))]
#[post("/components")]
pub async fn create_component(
    bom_service: web::Data<BomService>,
    component: web::Json<NewComponent>,
) -> Result<HttpResponse, ApiError> {
    let new_component =
        actix_web::web::block(move || bom_service.insert_component(component.into_inner()))
            .await??;
    Ok(HttpResponse::Created().json(new_component))
}
