diesel::table! {
    details (id) {
        id -> Int4,
        mean_radius -> Numeric,
        mass -> Numeric,
        population -> Nullable<Numeric>,
        planet_id -> Int4,
    }
}

diesel::table! {
    planets (id) {
        id -> Int4,
        name -> Varchar,
        #[sql_name = "type"]
        type_ -> Varchar,
    }
}

diesel::joinable!(details -> planets (planet_id));

diesel::allow_tables_to_appear_in_same_query!(details, planets,);
