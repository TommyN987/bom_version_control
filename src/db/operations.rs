use diesel::prelude::*;
use uuid::Uuid;

use crate::{
    domain::BOM,
    schema::{boms, boms_components, components},
};

use super::models::{db_bom::DbBOM, db_component::DbComponent};

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
