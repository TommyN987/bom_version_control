use diesel::{
    associations::Identifiable,
    deserialize::QueryableByName,
    prelude::{Insertable, Queryable},
    Selectable,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::schema::components;

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
pub struct Component {
    pub id: Uuid,
    pub name: String,
    pub part_number: String,
    pub description: Option<String>,
    pub supplier: String,
    pub price_value: f32,
    pub price_currency: String,
}
