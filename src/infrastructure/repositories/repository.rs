use uuid::Uuid;

use crate::infrastructure::{
    error::DatabaseError,
    models::{
        bom::BOM, bom_components::BomComponent, bom_version::BomVersion, component::Component,
    },
};

pub trait Repository: Send + Sync + 'static {
    #[allow(clippy::type_complexity)]
    fn find_all(&self) -> Result<Vec<(BOM, Vec<(Component, i32)>)>, DatabaseError>;

    fn find_by_id(&self, bom_id: Uuid) -> Result<(BOM, Vec<(Component, i32)>), DatabaseError>;

    fn insert(
        &self,
        new_bom: &BOM,
        new_bom_components: &[BomComponent],
        new_bom_version: &BomVersion,
    ) -> Result<(BOM, Vec<(Component, i32)>), DatabaseError>;

    fn update_and_archive(
        &self,
        bom_id: Uuid,
        updated_bom: &BOM,
        updated_bom_components: &[BomComponent],
        updated_bom_version: &BomVersion,
    ) -> Result<(BOM, Vec<(Component, i32)>), DatabaseError>;

    fn get_bom_versions_until_version(
        &self,
        bom_id: Uuid,
        version: i32,
    ) -> Result<Vec<BomVersion>, DatabaseError>;

    fn find_all_components(&self) -> Result<Vec<Component>, DatabaseError>;

    fn find_component_by_id(&self, component_id: Uuid) -> Result<Component, DatabaseError>;

    fn insert_component(&self, new_component: Component) -> Result<Component, DatabaseError>;

    fn update_component(&self, component: Component) -> Result<Component, DatabaseError>;

    fn search_components(&self, query_string: &str) -> Result<Vec<Component>, DatabaseError>;
}
