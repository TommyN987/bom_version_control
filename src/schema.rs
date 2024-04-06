// @generated automatically by Diesel CLI.

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
