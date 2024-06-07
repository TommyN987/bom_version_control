use chrono::{DateTime, Utc};
use diesel::{
    deserialize::FromSqlRow, expression::AsExpression, prelude::Insertable, sql_types::Jsonb,
    Associations, Identifiable, Queryable, Selectable,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::bom::BOM as Bom;

use crate::{domain::BOMChangeEvent, schema::bom_versions};

#[derive(
    Debug,
    PartialEq,
    Serialize,
    Deserialize,
    Identifiable,
    Selectable,
    Queryable,
    Insertable,
    Associations,
)]
#[diesel(belongs_to(Bom))]
#[diesel(table_name = bom_versions)]
pub struct BomVersion {
    pub id: Uuid,
    pub bom_id: Uuid,
    pub version: i32,
    pub changes: DbBomVersionChanges,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromSqlRow, AsExpression, PartialEq)]
#[serde(transparent)]
#[diesel(sql_type = Jsonb)]
/// Ideally we could have a DbBOMChangeEvent, but it was easier to just use the domain type here
pub struct DbBomVersionChanges(pub Vec<BOMChangeEvent>);

impl diesel::deserialize::FromSql<diesel::sql_types::Jsonb, diesel::pg::Pg>
    for DbBomVersionChanges
{
    fn from_sql(
        value: <diesel::pg::Pg as diesel::backend::Backend>::RawValue<'_>,
    ) -> diesel::deserialize::Result<Self> {
        let value = <serde_json::Value as diesel::deserialize::FromSql<
            diesel::sql_types::Jsonb,
            diesel::pg::Pg,
        >>::from_sql(value)?;
        Ok(serde_json::from_value(value)?)
    }
}

impl diesel::serialize::ToSql<diesel::sql_types::Jsonb, diesel::pg::Pg> for DbBomVersionChanges {
    fn to_sql(
        &self,
        out: &mut diesel::serialize::Output<'_, '_, diesel::pg::Pg>,
    ) -> diesel::serialize::Result {
        let value = serde_json::to_value(self)?;
        <serde_json::Value as diesel::serialize::ToSql<
        diesel::sql_types::Jsonb,
        diesel::pg::Pg,
    >>::to_sql(&value, &mut out.reborrow())
    }
}
