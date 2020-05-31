use crate::persistence::schema::users;

#[derive(Identifiable, Queryable)]
#[table_name = "users"]
pub struct UserEntity {
    pub id: i32,
    pub username: String,
    pub hash: String,
    pub first_name: String,
    pub last_name: String,
}

#[derive(Insertable)]
#[table_name = "users"]
pub struct NewUserEntity {
    pub username: String,
    pub hash: String,
    pub first_name: String,
    pub last_name: String,
}
