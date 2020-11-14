use diesel::prelude::*;

use crate::persistence::model::{NewUserEntity, UserEntity};
use crate::persistence::schema::users;

pub fn get_all(conn: &PgConnection) -> QueryResult<Vec<UserEntity>> {
    use crate::persistence::schema::users::dsl::*;

    users.load(conn)
}

pub fn get_user(username: &str, conn: &PgConnection) -> QueryResult<UserEntity> {
    users::table
        .filter(users::username.eq(username))
        .first(conn)
}

pub fn create(new_user: NewUserEntity, conn: &PgConnection) -> QueryResult<UserEntity> {
    use crate::persistence::schema::users::dsl::*;

    diesel::insert_into(users)
        .values(new_user)
        .get_result(conn)
}

pub fn update_password_hash(new_hash: String, conn: &PgConnection) -> QueryResult<usize> {
    use crate::persistence::schema::users::dsl::*;

    diesel::update(users)
        .set(hash.eq(new_hash))
        .execute(conn)
}
