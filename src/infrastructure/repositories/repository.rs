use uuid::Uuid;

use crate::infrastructure::{
    error::DatabaseError,
    models::{
        bom::BOM, bom_components::BomComponent, bom_version::BomVersion, component::Component,
    },
};

pub trait Repository {
    fn find_all(&self) -> Result<Vec<(BOM, Vec<(Component, i32)>)>, DatabaseError>;

    fn find_by_id(&self, bom_id: Uuid) -> Result<(BOM, Vec<(Component, i32)>), DatabaseError>;

    fn insert(
        &self,
        new_bom: &BOM,
        new_bom_components: &Vec<BomComponent>,
        new_bom_version: &BomVersion,
    ) -> Result<(BOM, Vec<(Component, i32)>), DatabaseError>;

    fn update(
        &self,
        bom_id: Uuid,
        updated_bom: &BOM,
        updated_bom_components: &Vec<BomComponent>,
        updated_bom_version: &BomVersion,
    ) -> Result<(BOM, Vec<(Component, i32)>), DatabaseError>;

    fn find_all_components(&self) -> Result<Vec<Component>, DatabaseError>;

    fn find_component_by_id(&self, component_id: Uuid) -> Result<Component, DatabaseError>;

    fn find_multiple_components_by_id(
        &self,
        component_ids: Vec<Uuid>,
    ) -> Result<Vec<Component>, DatabaseError>;

    fn insert_component(&self, new_component: Component) -> Result<Component, DatabaseError>;

    fn update_component(&self, component: Component) -> Result<Component, DatabaseError>;

    fn search_components(&self, query_string: &str) -> Result<Vec<Component>, DatabaseError>;
}
