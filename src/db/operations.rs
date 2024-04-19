use std::collections::HashMap;

use diesel::prelude::*;
use uuid::Uuid;

use crate::{
    domain::{Component, BOM},
    schema::{boms, boms_components, components},
};

use super::models::{db_bom::DbBOM, db_boms_component::DbBOMComponent, db_component::DbComponent};

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

pub fn insert_bom(
    conn: &mut PgConnection,
    new_bom: DbBOM,
    components: Vec<(Uuid, i32)>,
) -> Result<BOM, anyhow::Error> {
    conn.build_transaction().run(|conn| {
        let new_db_bom: DbBOM = diesel::insert_into(boms::table)
            .values(&new_bom)
            .get_result(conn)?;

        let mut comp_map: HashMap<Uuid, (Option<Component>, i32)> = HashMap::new();

        components.iter().for_each(|(id, quantity)| {
            comp_map.insert(*id, (None, *quantity));
        });

        let bom_components: Vec<DbBOMComponent> = components
            .iter()
            .map(|(component_id, quantity)| {
                DbBOMComponent::new(new_db_bom.id, *component_id, *quantity)
            })
            .collect();

        diesel::insert_into(boms_components::table)
            .values(&bom_components)
            .execute(conn)?;

        let comps: Vec<DbComponent> = components::table
            .filter(components::id.eq_any(components.iter().map(|(id, _)| *id)))
            .load(conn)?;

        comps.into_iter().for_each(|c| {
            comp_map
                .entry(c.id)
                .and_modify(|(comp, _)| *comp = Some(c.into()));
        });

        Ok(BOM::try_from((new_db_bom, comp_map))?)
    })
}

pub fn load_bom_with_components(
    conn: &mut PgConnection,
    bom_id: Uuid,
) -> Result<BOM, anyhow::Error> {
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
