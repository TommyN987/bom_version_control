use actix_web::{get, HttpResponse};

#[tracing::instrument(name = "Health check")]
#[get("/health_check")]
pub async fn health_check() -> HttpResponse {
    HttpResponse::Ok().finish()
}
