use diesel::{PgConnection, QueryDsl, RunQueryDsl};
use uuid::Uuid;

use crate::{
    infrastructure::{
        aliases::DbPool,
        error::DatabaseError,
        models::{bom::BOM, bom_components::BomComponent, bom_version::BOMVersion},
    },
    schema::{bom_versions, boms, boms_components},
};

pub struct BomRepository {
    pool: DbPool,
}

impl BomRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

impl BomRepository {
    pub fn find_all(&self) -> Result<Vec<BOM>, DatabaseError> {
        let mut conn = self.pool.get()?;

        Ok(boms::table.load::<BOM>(&mut conn)?)
    }

    pub fn find_by_id(&self, bom_id: Uuid) -> Result<BOM, DatabaseError> {
        let mut conn = self.pool.get()?;

        Ok(boms::table.find(bom_id).first::<BOM>(&mut conn)?)
    }

    pub fn insert(
        &self,
        new_bom: BOM,
        new_bom_components: Vec<BomComponent>,
        new_bom_version: BOMVersion,
    ) -> Result<(BOM, Vec<BomComponent>, BOMVersion), DatabaseError> {
        let mut conn = self.pool.get()?;
        conn.build_transaction().run(|conn| {
            let created_bom = self.insert_bom(new_bom, conn)?;
            let created_bom_version = self.insert_bom_version(new_bom_version, conn)?;
            let created_bom_components = self.insert_bom_components(new_bom_components, conn)?;

            Ok((created_bom, created_bom_components, created_bom_version))
        })
    }

    fn insert_bom(&self, new_bom: BOM, conn: &mut PgConnection) -> Result<BOM, DatabaseError> {
        Ok(diesel::insert_into(boms::table)
            .values(&new_bom)
            .get_result(conn)?)
    }

    fn insert_bom_version(
        &self,
        new_bom_version: BOMVersion,
        conn: &mut PgConnection,
    ) -> Result<BOMVersion, DatabaseError> {
        Ok(diesel::insert_into(bom_versions::table)
            .values(&new_bom_version)
            .get_result(conn)?)
    }

    fn insert_bom_components(
        &self,
        new_bom_components: Vec<BomComponent>,
        conn: &mut PgConnection,
    ) -> Result<Vec<BomComponent>, DatabaseError> {
        Ok(diesel::insert_into(boms_components::table)
            .values(&new_bom_components)
            .get_results(conn)?)
    }
}
