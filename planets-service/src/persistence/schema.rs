table! {
    details (id) {
        id -> Int4,
        mean_radius -> Numeric,
        mass -> Numeric,
        population -> Nullable<Numeric>,
        planet_id -> Int4,
    }
}

table! {
    planets (id) {
        id -> Int4,
        name -> Varchar,
        #[sql_name = "type"]
        type_ -> Varchar,
    }
}

joinable!(details -> planets (planet_id));

allow_tables_to_appear_in_same_query!(
    details,
    planets,
);
