table! {
    satellites (id) {
        id -> Int4,
        name -> Varchar,
        life_exists -> Varchar,
        first_spacecraft_landing_date -> Nullable<Date>,
        planet_id -> Int4,
    }
}
