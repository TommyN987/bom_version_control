use anyhow::anyhow;
use bom::{ValidationError, BOM};
use diesel::prelude::*;
use uuid::Uuid;

use crate::{
    domain::{bom, BOMChangeEvent, BOMChangeEventValidator},
    schema::{bom_versions, boms, boms_components, components},
};

use super::models::{
    db_bom::DbBOM, db_bom_version::BOMVersion, db_boms_component::DbBOMComponent,
    db_component::DbComponent,
};

pub fn find_components(conn: &mut PgConnection) -> Result<Vec<DbComponent>, diesel::result::Error> {
    components::table.load::<DbComponent>(conn)
}

pub fn find_component_by_id(
    conn: &mut PgConnection,
    component_id: Uuid,
) -> Result<DbComponent, diesel::result::Error> {
    components::table
        .find(component_id)
        .first::<DbComponent>(conn)
}

pub fn find_multiple_components(
    conn: &mut PgConnection,
    component_ids: &Vec<Uuid>,
) -> Result<Vec<DbComponent>, diesel::result::Error> {
    components::table
        .filter(components::id.eq_any(component_ids))
        .load::<DbComponent>(conn)
}

pub fn insert_component(
    conn: &mut PgConnection,
    new_component: DbComponent,
) -> Result<DbComponent, diesel::result::Error> {
    diesel::insert_into(components::table)
        .values(&new_component)
        .get_result(conn)
}

pub fn find_boms(conn: &mut PgConnection) -> Result<Vec<DbBOM>, diesel::result::Error> {
    boms::table.load::<DbBOM>(conn)
}

pub fn find_bom_by_id(
    conn: &mut PgConnection,
    bom_id: Uuid,
) -> Result<DbBOM, diesel::result::Error> {
    boms::table.find(bom_id).first::<DbBOM>(conn)
}

pub fn insert_bom(
    conn: &mut PgConnection,
    new_bom: DbBOM,
    boms_components: Vec<DbBOMComponent>,
    change_events: Vec<BOMChangeEvent>,
) -> Result<(DbBOM, Vec<DbBOMComponent>), anyhow::Error> {
    let mut bom_version = BOMVersion::from((&new_bom, &change_events));
    bom_version.version = 0;
    conn.build_transaction().run(|conn| {
        let new_db_bom: DbBOM = diesel::insert_into(boms::table)
            .values(&new_bom)
            .get_result(conn)?;

        let new_bom_compoonents: Vec<DbBOMComponent> = diesel::insert_into(boms_components::table)
            .values(&boms_components)
            .get_results(conn)?;

        diesel::insert_into(bom_versions::table)
            .values(bom_version)
            .execute(conn)?;

        Ok((new_db_bom, new_bom_compoonents))
    })
}

pub fn get_components_of_bom_by_id(
    conn: &mut PgConnection,
    bom_id: Uuid,
) -> Result<(Vec<DbBOMComponent>, Vec<DbComponent>), anyhow::Error> {
    let bom_components: Vec<DbBOMComponent> = boms_components::table
        .filter(boms_components::bom_id.eq(bom_id))
        .load::<DbBOMComponent>(conn)?;

    let components: Vec<DbComponent> = boms_components::table
        .inner_join(components::table.on(boms_components::component_id.eq(components::id)))
        .filter(boms_components::bom_id.eq(bom_id))
        .select(components::all_columns)
        .load(conn)?;
    Ok((bom_components, components))
}

pub fn update_and_archive_bom_by_id(
    conn: &mut PgConnection,
    bom_id: Uuid,
    change_events: Vec<BOMChangeEvent>,
) -> Result<(DbBOM, Vec<DbBOMComponent>, Vec<DbComponent>), anyhow::Error> {
    let db_bom: DbBOM = boms::table.find(bom_id).first(conn)?;

    let (bom_components, components) = get_components_of_bom_by_id(conn, bom_id)?;

    let db_bom_version: BOMVersion = (&db_bom, &change_events).into();
    let mut bom: BOM = (db_bom, bom_components, components).try_into()?;
    bom.increment_version();

    let _ = change_events
        .iter()
        .try_for_each(|event| -> Result<(), ValidationError> {
            bom.apply_change(event, BOMChangeEventValidator)?;
            Ok(())
        });

    let (new_db_bom, new_db_bom_components) = bom.into();

    let (updated_bom, updated_bom_components): (DbBOM, Vec<DbBOMComponent>) =
        conn.build_transaction().run(|conn| {
            let updated_bom: DbBOM = diesel::update(boms::table.find(bom_id))
                .set(new_db_bom)
                .get_result(conn)?;

            diesel::delete(boms_components::table.filter(boms_components::bom_id.eq(bom_id)))
                .execute(conn)?;

            let updated_bom_components: Vec<DbBOMComponent> =
                diesel::insert_into(boms_components::table)
                    .values(&new_db_bom_components)
                    .get_results(conn)?;

            let _ = diesel::insert_into(bom_versions::table)
                .values(db_bom_version)
                .execute(conn)
                .map_err(|e| anyhow!(e))?;

            Ok::<(DbBOM, Vec<DbBOMComponent>), anyhow::Error>((updated_bom, updated_bom_components))
        })?;

    let (_, components) = get_components_of_bom_by_id(conn, updated_bom.id)?;

    Ok((updated_bom, updated_bom_components, components))
}
