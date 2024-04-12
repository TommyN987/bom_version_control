use actix_web::{get, web, HttpResponse};
use uuid::Uuid;

use crate::{
    db::{operations::find_components, DbPool},
    domain::Component,
    errors::ApiError,
};

#[get("/components")]
pub async fn get_components(pool: web::Data<DbPool>) -> Result<HttpResponse, ApiError> {
    let mut conn = pool
        .get()
        .map_err(|e| ApiError::new_internal(e.to_string()))?;
    let components: Vec<Component> = actix_web::web::block(move || find_components(&mut conn))
        .await??
        .into_iter()
        .map(|c| c.into())
        .collect();
    Ok(HttpResponse::Ok().json(components))
}

#[get("/components/{id}")]
pub async fn get_component_by_id(
    pool: web::Data<DbPool>,
    id: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let mut conn = pool
        .get()
        .map_err(|e| ApiError::new_internal(e.to_string()))?;
    let component: Component =
        actix_web::web::block(move || crate::db::operations::find_component_by_id(&mut conn, *id))
            .await??
            .into();
    Ok(HttpResponse::Ok().json(component))
}
