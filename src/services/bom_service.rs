use std::sync::Arc;

use uuid::Uuid;

use crate::{
    domain::{
        newtypes::{new_bom::NewBOM, new_component::NewComponent},
        validation::BOMChangeEventValidator,
        BOMChangeEvent, BOMDiff, BomVersion, Component as DomainComponent, CountedComponent, BOM,
    },
    infrastructure::{
        models::{
            bom::BOM as DbBOM, bom_components::BomComponent,
            bom_version::BomVersion as DbBomVersion,
        },
        repositories::repository::Repository,
    },
};

use super::error::ServiceError;

pub enum UpdateOperation {
    Incremental,
    Revert,
}

pub struct BomService {
    repo: Arc<dyn Repository>,
}

impl BomService {
    pub fn new(repo: Arc<dyn Repository>) -> Self {
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

    pub fn find_bom_by_version_and_id(
        &self,
        bom_id: Uuid,
        version: i32,
    ) -> Result<BOM, ServiceError> {
        let mut bom: BOM = self.repo.find_by_id(bom_id)?.into();

        if bom.version < version {
            return Err(ServiceError::InvalidData(format!(
                "Version not found. Your latest version is {}",
                &bom.version
            )));
        }

        let versions = self.fetch_bom_versions_until_version(bom_id, version)?;

        bom.clean_for_revert();
        if let Some(version) = versions.last() {
            bom.version = version.version + 1;
        }

        for version in versions.iter() {
            for change_event in version.changes.iter() {
                bom.apply_change(change_event, BOMChangeEventValidator)?;
            }
        }

        Ok(bom)
    }

    pub fn insert_bom(&self, new_bom: NewBOM) -> Result<BOM, ServiceError> {
        let bom: BOM = BOM::try_from(&new_bom)?;

        let new_bom_components = self.transform_counted_components(&bom.id, &bom.components);

        let new_bom_version: DbBomVersion =
            BomVersion::new(&bom.id, 0, Box::new(new_bom.events)).try_into()?;

        let (bom, components) =
            self.repo
                .insert(&bom.into(), &new_bom_components, &new_bom_version)?;

        println!("{:?}", components);

        Ok(BOM::from((bom, components)))
    }

    pub fn update_bom(
        &self,
        bom_id: Uuid,
        change_events: Box<Vec<BOMChangeEvent>>,
        operation: UpdateOperation,
    ) -> Result<BOM, ServiceError> {
        let mut bom = BOM::from(self.repo.find_by_id(bom_id)?);

        if let UpdateOperation::Revert = operation {
            bom.clean_for_revert();
        }

        bom.increment_version();

        let _ = change_events
            .iter()
            .try_for_each(|event| -> Result<(), ServiceError> {
                bom.apply_change(event, BOMChangeEventValidator)?;
                Ok(())
            });

        let new_bom_components = self.transform_counted_components(&bom_id, &bom.components);

        let new_bom_version: DbBomVersion =
            BomVersion::new(&bom.id, bom.version, change_events).try_into()?;

        let bom: DbBOM = bom.into();

        let (updated_bom, components) =
            self.repo
                .update_and_archive(bom_id, &bom, &new_bom_components, &new_bom_version)?;

        Ok(BOM::from((updated_bom, components)))
    }

    pub fn revert_bom_to_version(&self, bom_id: Uuid, version: i32) -> Result<BOM, ServiceError> {
        let versions = self.fetch_bom_versions_until_version(bom_id, version)?;

        let mut change_events: Box<Vec<BOMChangeEvent>> = Box::default();

        versions.into_iter().for_each(|version| {
            version.changes.into_iter().for_each(|change_event| {
                change_events.push(change_event);
            })
        });

        self.update_bom(bom_id, change_events, UpdateOperation::Revert)
    }

    pub fn get_bom_diff(&self, bom_id: Uuid, from: i32, to: i32) -> Result<BOMDiff, ServiceError> {
        let versions = self.fetch_bom_versions_until_version(bom_id, to)?;

        let mut events_until_starting_bom = vec![];
        let mut events_until_ending_bom = vec![];

        versions.into_iter().enumerate().for_each(|(i, version)| {
            if i <= (from - 1) as usize {
                version.changes.into_iter().for_each(|change_event| {
                    events_until_starting_bom.push(change_event);
                })
            } else {
                version.changes.into_iter().for_each(|change_event| {
                    events_until_ending_bom.push(change_event);
                })
            }
        });

        let starting_bom = BOM::try_from(&NewBOM::new(events_until_starting_bom))?;

        let diff = BOMDiff::from((&starting_bom, &events_until_ending_bom));

        Ok(diff)
    }
}

impl BomService {
    pub fn find_all_components(&self) -> Result<Vec<DomainComponent>, ServiceError> {
        Ok(self
            .repo
            .find_all_components()?
            .into_iter()
            .map(DomainComponent::from)
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
            .map(DomainComponent::from)
            .collect())
    }
}

impl BomService {
    fn transform_counted_components(
        &self,
        bom_id: &Uuid,
        counted_components: &[CountedComponent],
    ) -> Vec<BomComponent> {
        counted_components
            .iter()
            .map(|counted_component| BomComponent::from((bom_id, counted_component)))
            .collect()
    }

    fn fetch_bom_versions_until_version(
        &self,
        bom_id: Uuid,
        version: i32,
    ) -> Result<Vec<BomVersion>, ServiceError> {
        self.repo
            .get_bom_versions_until_version(bom_id, version)?
            .into_iter()
            .map(|version| BomVersion::try_from(version).map_err(ServiceError::from))
            .collect::<Result<Vec<BomVersion>, ServiceError>>()
    }
}
