use diesel;
use diesel::prelude::*;

use crate::db::{DetailsEntity, NewDetailsEntity, NewPlanetEntity, PlanetEntity};
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

pub fn create(new_planet: NewPlanetEntity, mut new_details_entity: NewDetailsEntity, connection: &PgConnection) -> QueryResult<PlanetEntity> {
    let result: QueryResult<PlanetEntity> = diesel::insert_into(planets::table)
        .values(new_planet)
        .get_result(connection);

    let new_planet_id = result.as_ref().ok().expect("Can't get created planet").id;

    new_details_entity.planet_id = new_planet_id;

    diesel::insert_into(details::table)
        .values(new_details_entity)
        .execute(connection);

    result
}
