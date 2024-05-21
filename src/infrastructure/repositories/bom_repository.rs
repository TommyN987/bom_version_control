use std::vec;

use diesel::{ExpressionMethods, JoinOnDsl, PgConnection, QueryDsl, RunQueryDsl};
use uuid::Uuid;

use crate::{
    infrastructure::{
        aliases::DbPool,
        error::DatabaseError,
        models::{
            bom::BOM, bom_components::BomComponent, bom_version::BomVersion, component::Component,
        },
        repositories::repository::Repository,
    },
    schema::{bom_versions, boms, boms_components, components},
};

pub struct BomRepository {
    pool: DbPool,
}

impl BomRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

impl Repository for BomRepository {
    fn find_all(&self) -> Result<Vec<(BOM, Vec<Component>)>, DatabaseError> {
        let mut conn = self.pool.get()?;

        let mut result: Vec<(BOM, Vec<Component>)> = vec![];
        let boms = self.find_all_boms(&mut conn)?;

        for bom in boms {
            let components = self.find_components_of_bom_by_bom_id(bom.id, &mut conn)?;
            result.push((bom, components));
        }

        Ok(result)
    }

    fn find_by_id(&self, bom_id: Uuid) -> Result<(BOM, Vec<Component>), DatabaseError> {
        let mut conn = self.pool.get()?;

        let bom = self.find_bom_by_id(bom_id, &mut conn)?;
        let components = self.find_components_of_bom_by_bom_id(bom_id, &mut conn)?;

        Ok((bom, components))
    }

    fn insert(
        &self,
        new_bom: &BOM,
        new_bom_components: &Vec<BomComponent>,
        new_bom_version: &BomVersion,
    ) -> Result<(BOM, Vec<BomComponent>), DatabaseError> {
        let mut conn = self.pool.get()?;
        conn.build_transaction().run(|conn| {
            let created_bom = self.insert_bom(new_bom, conn)?;
            let _ = self.insert_bom_version(new_bom_version, conn)?;
            let created_bom_components = self.insert_bom_components(new_bom_components, conn)?;

            Ok((created_bom, created_bom_components))
        })
    }

    fn update(
        &self,
        bom_id: Uuid,
        updated_bom: &BOM,
        updated_bom_components: &Vec<BomComponent>,
        updated_bom_version: &BomVersion,
    ) -> Result<(BOM, Vec<Component>), DatabaseError> {
        let mut conn = self.pool.get()?;

        conn.build_transaction().run(|conn| {
            let updated_bom = self.update_bom_by_id(bom_id, updated_bom, conn)?;
            self.delete_bom_components_by_bom_id(bom_id, conn)?;
            let _ = self.insert_bom_components(updated_bom_components, conn)?;
            let _ = self.insert_bom_version(updated_bom_version, conn)?;
            let components = self.find_components_of_bom_by_bom_id(bom_id, conn)?;

            Ok((updated_bom, components))
        })
    }

    fn find_all_components(&self) -> Result<Vec<Component>, DatabaseError> {
        let mut conn = self.pool.get()?;

        Ok(components::table.load::<Component>(&mut conn)?)
    }

    fn find_component_by_id(&self, component_id: Uuid) -> Result<Component, DatabaseError> {
        let mut conn = self.pool.get()?;

        Ok(components::table
            .find(component_id)
            .first::<Component>(&mut conn)?)
    }

    fn insert_component(&self, new_component: Component) -> Result<Component, DatabaseError> {
        let mut conn = self.pool.get()?;

        Ok(diesel::insert_into(components::table)
            .values(&new_component)
            .get_result(&mut conn)?)
    }

    fn update_component(&self, component: Component) -> Result<Component, DatabaseError> {
        let mut conn = self.pool.get()?;

        Ok(diesel::update(components::table.find(component.id))
            .set(&component)
            .get_result(&mut conn)?)
    }

    fn search_components(&self, query_string: &str) -> Result<Vec<Component>, DatabaseError> {
        let mut conn = self.pool.get()?;

        let sql = format!(
        "SELECT *
        FROM components
        WHERE to_tsvector('english', coalesce(name, '') || ' ' || coalesce(part_number, '') || ' ' || coalesce(description, '') || ' ' || coalesce(supplier, ''))
        @@ plainto_tsquery('english', '{}')",
        query_string.to_lowercase()
        );

        Ok(diesel::sql_query(sql).load::<Component>(&mut conn)?)
    }
}

impl BomRepository {
    fn find_all_boms(&self, conn: &mut PgConnection) -> Result<Vec<BOM>, DatabaseError> {
        Ok(boms::table.load::<BOM>(conn)?)
    }

    fn find_bom_by_id(&self, bom_id: Uuid, conn: &mut PgConnection) -> Result<BOM, DatabaseError> {
        Ok(boms::table.find(bom_id).first::<BOM>(conn)?)
    }

    fn find_components_of_bom_by_bom_id(
        &self,
        bom_id: Uuid,
        conn: &mut PgConnection,
    ) -> Result<Vec<Component>, DatabaseError> {
        Ok(boms_components::table
            .inner_join(components::table.on(components::id.eq(components::id)))
            .filter(boms_components::bom_id.eq(bom_id))
            .select(components::all_columns)
            .load(conn)?)
    }

    fn insert_bom(&self, new_bom: &BOM, conn: &mut PgConnection) -> Result<BOM, DatabaseError> {
        Ok(diesel::insert_into(boms::table)
            .values(new_bom)
            .get_result(conn)?)
    }

    fn insert_bom_version(
        &self,
        new_bom_version: &BomVersion,
        conn: &mut PgConnection,
    ) -> Result<BomVersion, DatabaseError> {
        Ok(diesel::insert_into(bom_versions::table)
            .values(new_bom_version)
            .get_result(conn)?)
    }

    fn insert_bom_components(
        &self,
        new_bom_components: &Vec<BomComponent>,
        conn: &mut PgConnection,
    ) -> Result<Vec<BomComponent>, DatabaseError> {
        Ok(diesel::insert_into(boms_components::table)
            .values(new_bom_components)
            .get_results(conn)?)
    }

    fn update_bom_by_id(
        &self,
        bom_id: Uuid,
        updated_bom: &BOM,
        conn: &mut PgConnection,
    ) -> Result<BOM, DatabaseError> {
        Ok(diesel::update(boms::table.find(bom_id))
            .set(updated_bom)
            .get_result(conn)?)
    }

    fn delete_bom_components_by_bom_id(
        &self,
        bom_id: Uuid,
        conn: &mut PgConnection,
    ) -> Result<(), DatabaseError> {
        diesel::delete(boms_components::table.filter(boms_components::bom_id.eq(bom_id)))
            .execute(conn)?;

        Ok(())
    }
}
