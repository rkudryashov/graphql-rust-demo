diesel::table! {
    users (id) {
        id -> Int4,
        username -> Varchar,
        hash -> Varchar,
        first_name -> Varchar,
        last_name -> Varchar,
        role -> Varchar,
    }
}
