use diesel::prelude::*;

use crate::persistence::model::{NewUserEntity, UserEntity};
use crate::persistence::schema::users;

pub fn create(new_user: NewUserEntity, conn: &PgConnection) -> QueryResult<UserEntity> {
    use crate::persistence::schema::users::dsl::*;

    diesel::insert_into(users)
        .values(new_user)
        .get_result(conn)
}

pub fn get_hash(username: &str, conn: &PgConnection) -> QueryResult<String> {
    users::table
        .filter(users::username.eq(username))
        .select(users::hash)
        .first(conn)
}
