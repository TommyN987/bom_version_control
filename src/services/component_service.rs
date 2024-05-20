use uuid::Uuid;

use crate::{
    domain::{newtypes::new_component::NewComponent, Component as DomainComponent, Price},
    infrastructure::{
        aliases::DbPool, error::DatabaseError, models::component::Component as DbComponent,
        repositories::component_repository::ComponentRepository,
    },
};

pub struct ComponentService {
    repo: ComponentRepository,
}

impl ComponentService {
    pub fn new(pool: DbPool) -> Self {
        Self {
            repo: ComponentRepository::new(pool),
        }
    }

    pub fn find_all(&self) -> Result<Vec<DomainComponent>, DatabaseError> {
        Ok(self
            .repo
            .find_all()?
            .into_iter()
            .map(|c| DomainComponent::from(c))
            .collect())
    }

    pub fn find_by_id(&self, component_id: Uuid) -> Result<DomainComponent, DatabaseError> {
        Ok(DomainComponent::from(self.repo.find_by_id(component_id)?))
    }

    pub fn find_multiple(
        &self,
        component_ids: Vec<Uuid>,
    ) -> Result<Vec<DomainComponent>, DatabaseError> {
        Ok(self
            .repo
            .find_multiple(component_ids)?
            .into_iter()
            .map(|c| DomainComponent::from(c))
            .collect())
    }

    pub fn insert(&self, new_component: NewComponent) -> Result<DomainComponent, DatabaseError> {
        Ok(DomainComponent::from(
            self.repo.insert(new_component.into())?,
        ))
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
