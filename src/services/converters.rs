use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::{
    domain::{
        error::DomainError, newtypes::new_component::NewComponent, BomVersion as DomainBomVersion,
        Component as DomainComponent, CountedComponent, Price, BOM,
    },
    infrastructure::models::{
        bom::BOM as DbBOM, bom_version::BomVersion as DbBomVersion,
        component::Component as DbComponent,
    },
};

impl From<(DbBOM, Vec<(DbComponent, i32)>)> for BOM {
    fn from(value: (DbBOM, Vec<(DbComponent, i32)>)) -> Self {
        let (bom, components) = value;
        Self {
            id: bom.id,
            name: bom.name,
            version: bom.version,
            description: bom.description,
            components: components
                .into_iter()
                .map(|(component, qty)| CountedComponent::new(component.into(), qty))
                .collect(),
        }
    }
}

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

impl From<DomainComponent> for DbComponent {
    fn from(value: DomainComponent) -> Self {
        Self {
            id: value.id,
            name: value.name,
            part_number: value.part_number,
            description: value.description,
            supplier: value.supplier,
            price_value: value.price.value,
            price_currency: value.price.currency,
        }
    }
}

impl From<BOM> for DbBOM {
    fn from(value: BOM) -> Self {
        Self {
            id: value.id,
            name: value.name,
            version: value.version,
            description: value.description,
            created_at: DateTime::<Utc>::from(Utc::now()),
            updated_at: DateTime::<Utc>::from(Utc::now()),
        }
    }
}

impl TryFrom<DomainBomVersion> for DbBomVersion {
    type Error = DomainError;

    fn try_from(value: DomainBomVersion) -> Result<Self, Self::Error> {
        Ok(Self {
            id: value.id,
            bom_id: value.bom_id,
            version: value.version,
            changes: serde_json::to_value(value.changes)
                .map_err(|e| DomainError::ConversionError(e.to_string()))?,
            created_at: DateTime::<Utc>::from(Utc::now()),
        })
    }
}
