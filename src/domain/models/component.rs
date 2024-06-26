use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Price {
    pub value: f32,
    pub currency: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Component {
    pub id: Uuid,
    pub name: String,
    pub part_number: String,
    pub description: Option<String>,
    pub supplier: String,
    pub price: Price,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CountedComponent {
    pub component: Component,
    pub quantity: i32,
}

impl CountedComponent {
    pub fn new(component: Component, quantity: i32) -> Self {
        Self {
            component,
            quantity,
        }
    }
}
