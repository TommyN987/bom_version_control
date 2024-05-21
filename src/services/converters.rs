use uuid::Uuid;

use crate::{
    domain::{newtypes::new_component::NewComponent, Component as DomainComponent, Price},
    infrastructure::models::component::Component as DbComponent,
};

impl From<NewComponent> for DbComponent {
    fn from(value: NewComponent) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: value.name,
            part_number: value.part_number,
            description: value.description,
            supplier: value.supplier,
            price_value: value.price.value,
            price_currency: value.price.currency,
        }
    }
}

impl From<DbComponent> for DomainComponent {
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
