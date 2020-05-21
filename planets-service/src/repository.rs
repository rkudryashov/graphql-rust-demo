use diesel;
use diesel::prelude::*;

use crate::db::{DetailsEntity, NewPlanetEntity, PlanetEntity};
use crate::schema::details;
use crate::schema::planets;

pub fn all(connection: &PgConnection) -> QueryResult<Vec<(PlanetEntity, DetailsEntity)>> {
    planets::table
        .inner_join(details::table)
        .load(connection)
}

pub fn get(id: i32, connection: &PgConnection) -> QueryResult<PlanetEntity> {
    planets::table.find(id).get_result::<PlanetEntity>(connection)
}

pub fn create(new_planet: NewPlanetEntity, connection: &PgConnection) -> QueryResult<PlanetEntity> {
    diesel::insert_into(planets::table)
        .values(new_planet)
        .get_result(connection)
}
