// @generated automatically by Diesel CLI.

diesel::table! {
    bom_versions (id) {
        id -> Uuid,
        bom_id -> Uuid,
        version -> Int4,
        changes -> Jsonb,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    boms (id) {
        id -> Uuid,
        name -> Varchar,
        description -> Nullable<Text>,
        version -> Int4,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    boms_components (bom_id, component_id) {
        bom_id -> Uuid,
        component_id -> Uuid,
        quantity -> Int4,
    }
}

diesel::table! {
    components (id) {
        id -> Uuid,
        name -> Varchar,
        part_number -> Varchar,
        description -> Nullable<Text>,
        supplier -> Varchar,
        price_value -> Float4,
        price_currency -> Varchar,
    }
}

diesel::joinable!(bom_versions -> boms (bom_id));
diesel::joinable!(boms_components -> boms (bom_id));
diesel::joinable!(boms_components -> components (component_id));

diesel::allow_tables_to_appear_in_same_query!(bom_versions, boms, boms_components, components,);
