use crate::infrastructure::models::bom_version::DbBomVersionChanges;
use chrono::Utc;
use uuid::Uuid;

use crate::{
    domain::{
        error::DomainError,
        newtypes::{new_bom::NewBOM, new_component::NewComponent},
        validation::BOMChangeEventValidator,
        BOMChangeEvent, BomVersion as DomainBomVersion, Component as DomainComponent,
        CountedComponent, Price, BOM,
    },
    infrastructure::models::{
        bom::BOM as DbBOM, bom_components::BomComponent, bom_version::BomVersion as DbBomVersion,
        component::Component as DbComponent,
    },
};

/**********************************************************
****            Newtypes -> Domain models             *****
**********************************************************/

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

impl TryFrom<&NewBOM> for BOM {
    type Error = DomainError;

    fn try_from(value: &NewBOM) -> Result<Self, Self::Error> {
        if value.events.is_empty() {
            return Err(DomainError::ValidationError(
                "Unable to construct BOM without input".to_string(),
            ));
        }

        // Check if events include a NameChanged event
        if !value
            .events
            .iter()
            .any(|event| matches!(event, BOMChangeEvent::NameChanged(_)))
        {
            return Err(DomainError::ValidationError(
                "Creating a BOM without name is not allowed. Please provide a name".to_string(),
            ));
        }

        let mut bom = BOM::default();
        for event in value.events.iter() {
            bom.apply_change(event, BOMChangeEventValidator)?;
        }
        Ok(bom)
    }
}

/**********************************************************
****    Database BOM models <-> Domain BOM models    ******
**********************************************************/

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
            created_at: bom.created_at,
            updated_at: bom.updated_at,
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
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

impl From<DomainBomVersion> for DbBomVersion {
    fn from(value: DomainBomVersion) -> Self {
        Self {
            id: value.id,
            bom_id: value.bom_id,
            version: value.version,
            changes: DbBomVersionChanges(value.changes),
            created_at: Utc::now(),
        }
    }
}

impl From<DbBomVersion> for DomainBomVersion {
    fn from(value: DbBomVersion) -> Self {
        Self {
            id: value.id,
            bom_id: value.bom_id,
            version: value.version,
            changes: value.changes.0,
            created_at: value.created_at,
        }
    }
}

/**********************************************************
****     Database Component <-> Domain Component     ******
**********************************************************/

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

impl From<(&Uuid, &CountedComponent)> for BomComponent {
    fn from(value: (&Uuid, &CountedComponent)) -> Self {
        Self {
            bom_id: *value.0,
            component_id: value.1.component.id,
            quantity: value.1.quantity,
        }
    }
}
