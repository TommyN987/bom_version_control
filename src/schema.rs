// @generated automatically by Diesel CLI.

diesel::table! {
    boms (id) {
        id -> Uuid,
        name -> Varchar,
        description -> Nullable<Text>,
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
        price_value -> Int4,
        price_currency -> Varchar,
    }
}

diesel::joinable!(boms_components -> boms (bom_id));
diesel::joinable!(boms_components -> components (component_id));

diesel::allow_tables_to_appear_in_same_query!(boms, boms_components, components,);
