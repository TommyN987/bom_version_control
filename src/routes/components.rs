use std::fmt::{self, Display, Formatter};

use actix_web::{get, post, web, HttpResponse};
use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    db::{
        models::db_component::DbComponent,
        operations::{find_components, insert_component},
        DbPool,
    },
    domain::{Component, Price},
};

use super::ApiError;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct NewComponent {
    name: String,
    part_number: String,
    description: Option<String>,
    supplier: String,
    price: Price,
}

impl NewComponent {
    pub fn new(
        name: String,
        part_number: String,
        description: Option<String>,
        supplier: String,
        price: Price,
    ) -> Self {
        Self {
            name,
            part_number,
            description,
            supplier,
            price,
        }
    }
}

impl Display for NewComponent {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "NewComponent {{ name: {}, part_number: {}, description: {:?}, supplier: {}, price: {} {} }}",
            self.name, self.part_number, self.description, self.supplier, self.price.value, self.price.currency
        )
    }
}

impl TryFrom<NewComponent> for DbComponent {
    type Error = ApiError;

    fn try_from(value: NewComponent) -> Result<Self, Self::Error> {
        Ok(Self {
            id: Uuid::new_v4(),
            name: value.name,
            part_number: value.part_number,
            description: value.description,
            supplier: value.supplier,
            price_value: value.price.value,
            price_currency: value.price.currency,
        })
    }
}

#[tracing::instrument(name = "Getting all components", skip(pool), fields(request_id = %Uuid::new_v4()))]
#[get("/components")]
pub async fn get_components(pool: web::Data<DbPool>) -> Result<HttpResponse, ApiError> {
    let mut conn = pool.get().map_err(|e| anyhow!(e))?;
    let components: Vec<Component> = actix_web::web::block(move || find_components(&mut conn))
        .await??
        .into_iter()
        .map(|c| c.into())
        .collect();
    Ok(HttpResponse::Ok().json(components))
}

#[tracing::instrument(name = "Getting a component by id", skip(pool), fields(request_id = %Uuid::new_v4(), id = %id))]
#[get("/components/{id}")]
pub async fn get_component_by_id(
    pool: web::Data<DbPool>,
    id: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let mut conn = pool.get().map_err(|e| anyhow!(e))?;
    let component: Component =
        actix_web::web::block(move || crate::db::operations::find_component_by_id(&mut conn, *id))
            .await??
            .into();
    Ok(HttpResponse::Ok().json(component))
}

#[tracing::instrument(name = "Creating a component", skip(pool), fields(request_id = %Uuid::new_v4(), name = %component.name, part_number = %component.part_number, supplier = %component.supplier, price = %component.price.value, currency = %component.price.currency))]
#[post("/components")]
pub async fn create_component(
    pool: web::Data<DbPool>,
    component: web::Json<NewComponent>,
) -> Result<HttpResponse, ApiError> {
    let mut conn = pool.get().map_err(|e| anyhow!(e))?;
    let new_component: DbComponent = DbComponent::try_from(component.into_inner())?;
    let result =
        actix_web::web::block(move || insert_component(&mut conn, new_component)).await??;
    Ok(HttpResponse::Created().json(Component::from(result)))
}
