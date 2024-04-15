use diesel::prelude::{Insertable, Queryable};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    domain::{Component, Price},
    schema::components,
};

#[derive(Debug, PartialEq, Deserialize, Insertable, Queryable)]
#[diesel(table_name = components)]
pub struct DbComponent {
    pub id: Uuid,
    pub name: String,
    pub part_number: String,
    pub description: Option<String>,
    pub supplier: String,
    pub price_value: i32,
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