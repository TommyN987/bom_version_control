use diesel::{
    associations::Identifiable,
    deserialize::QueryableByName,
    prelude::{Insertable, Queryable},
    Selectable,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    domain::{Component, Price},
    schema::components,
};

#[derive(
    Debug,
    PartialEq,
    Serialize,
    Deserialize,
    Identifiable,
    Selectable,
    Insertable,
    Queryable,
    QueryableByName,
)]
#[diesel(table_name = components)]
pub struct DbComponent {
    pub id: Uuid,
    pub name: String,
    pub part_number: String,
    pub description: Option<String>,
    pub supplier: String,
    pub price_value: f32,
    pub price_currency: String,
}

impl From<DbComponent> for Component {
    fn from(value: DbComponent) -> Self {
        Self {
            id: value.id,
            name: value.name,
            part_number: value.part_number,
            description: value.description,
            supplier: value.supplier,
            price: Price {
                value: value.price_value,
                currency: value.price_currency,
            },
        }
    }
}
