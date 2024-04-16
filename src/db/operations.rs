use diesel::prelude::*;
use uuid::Uuid;

use crate::{
    domain::BOM,
    schema::{boms, boms_components, components},
};

use super::models::{db_bom::DbBOM, db_component::DbComponent};

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

pub fn insert_component(
    conn: &mut PgConnection,
    new_component: DbComponent,
) -> Result<DbComponent, diesel::result::Error> {
    diesel::insert_into(components::table)
        .values(&new_component)
        .get_result(conn)
}

pub fn load_bom_with_components(
    conn: &mut PgConnection,
    bom_id: Uuid,
) -> Result<BOM, diesel::result::Error> {
    let db_bom: DbBOM = boms::table.find(bom_id).first::<DbBOM>(conn)?;

    let components: Vec<DbComponent> = boms_components::table
        .inner_join(components::table.on(boms_components::component_id.eq(components::id)))
        .filter(boms_components::bom_id.eq(bom_id))
        .select(components::all_columns)
        .load(conn)?;

    let quantities: Vec<i32> = boms_components::table
        .filter(boms_components::bom_id.eq(bom_id))
        .select(boms_components::quantity)
        .load::<i32>(conn)?;

    Ok(BOM::from((db_bom, components, quantities)))
}
