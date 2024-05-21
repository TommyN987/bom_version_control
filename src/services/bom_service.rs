use uuid::Uuid;

use crate::{
    domain::{newtypes::new_component::NewComponent, Component as DomainComponent},
    infrastructure::{error::DatabaseError, repositories::repository::Repository},
};

pub struct BomService {
    repo: Box<dyn Repository>,
}

impl BomService {
    pub fn new(repo: Box<dyn Repository>) -> Self {
        Self { repo }
    }
}

impl BomService {
    pub fn find_all_components(&self) -> Result<Vec<DomainComponent>, DatabaseError> {
        Ok(self
            .repo
            .find_all_components()?
            .into_iter()
            .map(|c| DomainComponent::from(c))
            .collect())
    }

    pub fn find_component_by_id(
        &self,
        component_id: Uuid,
    ) -> Result<DomainComponent, DatabaseError> {
        Ok(DomainComponent::from(
            self.repo.find_component_by_id(component_id)?,
        ))
    }

    pub fn insert_component(
        &self,
        new_component: NewComponent,
    ) -> Result<DomainComponent, DatabaseError> {
        Ok(DomainComponent::from(
            self.repo.insert_component(new_component.into())?,
        ))
    }
}
