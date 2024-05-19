use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use uuid::Uuid;

use crate::{
    infrastructure::{aliases::DbPool, error::DatabaseError, models::component::Component},
    schema::components,
};

pub struct ComponentRepository {
    pool: DbPool,
}

impl ComponentRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

impl ComponentRepository {
    pub fn find_all(&self) -> Result<Vec<Component>, DatabaseError> {
        let mut conn = self.pool.get()?;

        Ok(components::table.load::<Component>(&mut conn)?)
    }

    pub fn find_by_id(&self, component_id: Uuid) -> Result<Component, DatabaseError> {
        let mut conn = self.pool.get()?;

        Ok(components::table
            .find(component_id)
            .first::<Component>(&mut conn)?)
    }

    pub fn find_multiple(&self, component_ids: Vec<Uuid>) -> Result<Vec<Component>, DatabaseError> {
        let mut conn = self.pool.get()?;

        Ok(components::table
            .filter(components::id.eq_any(component_ids))
            .load::<Component>(&mut conn)?)
    }

    pub fn insert(&self, new_component: Component) -> Result<Component, DatabaseError> {
        let mut conn = self.pool.get()?;

        Ok(diesel::insert_into(components::table)
            .values(&new_component)
            .get_result(&mut conn)?)
    }

    pub fn search(&self, query_string: &str) -> Result<Vec<Component>, DatabaseError> {
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
