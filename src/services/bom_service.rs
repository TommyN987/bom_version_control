use uuid::Uuid;

use crate::{
    domain::{
        newtypes::{new_bom::NewBOM, new_component::NewComponent},
        BomVersion, Component as DomainComponent, BOM,
    },
    infrastructure::{
        models::bom_components::BomComponent, models::bom_version::BomVersion as DbBomVersion,
        repositories::repository::Repository,
    },
};

use super::error::ServiceError;

pub struct BomService {
    repo: Box<dyn Repository>,
}

impl BomService {
    pub fn new(repo: Box<dyn Repository>) -> Self {
        Self { repo }
    }
}

impl BomService {
    pub fn find_all_boms(&self) -> Result<Vec<BOM>, ServiceError> {
        Ok(self
            .repo
            .find_all()?
            .into_iter()
            .map(|(bom, components)| BOM::from((bom, components)))
            .collect())
    }

    pub fn find_bom_by_id(&self, bom_id: Uuid) -> Result<BOM, ServiceError> {
        Ok(self.repo.find_by_id(bom_id)?.into())
    }

    pub fn insert_bom(&self, new_bom: NewBOM) -> Result<BOM, ServiceError> {
        let bom: BOM = BOM::try_from(&new_bom)?;
        let new_bom_components: Vec<BomComponent> = bom
            .components
            .iter()
            .map(|counted_component| BomComponent {
                bom_id: *&bom.id,
                component_id: counted_component.component.id,
                quantity: counted_component.quantity,
            })
            .collect();
        let new_bom_version: DbBomVersion =
            BomVersion::new(&bom.id, *&bom.version, new_bom.events).try_into()?;

        let (bom, components) =
            self.repo
                .insert(&bom.into(), &new_bom_components, &new_bom_version)?;

        Ok(BOM::from((bom, components)))
    }
}

impl BomService {
    pub fn find_all_components(&self) -> Result<Vec<DomainComponent>, ServiceError> {
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
    ) -> Result<DomainComponent, ServiceError> {
        Ok(DomainComponent::from(
            self.repo.find_component_by_id(component_id)?,
        ))
    }

    pub fn insert_component(
        &self,
        new_component: NewComponent,
    ) -> Result<DomainComponent, ServiceError> {
        Ok(DomainComponent::from(
            self.repo.insert_component(new_component.into())?,
        ))
    }

    pub fn update_component(
        &self,
        updated_component: DomainComponent,
    ) -> Result<DomainComponent, ServiceError> {
        Ok(DomainComponent::from(
            self.repo.update_component(updated_component.into())?,
        ))
    }

    pub fn search_components(
        &self,
        query_string: &str,
    ) -> Result<Vec<DomainComponent>, ServiceError> {
        Ok(self
            .repo
            .search_components(query_string)?
            .into_iter()
            .map(|c| DomainComponent::from(c))
            .collect())
    }
}
