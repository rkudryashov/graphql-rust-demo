use diesel::prelude::*;

use crate::persistence::model::{DetailsEntity, NewDetailsEntity, NewPlanetEntity, PlanetEntity};
use crate::persistence::schema::{details, planets};

pub fn all(conn: &PgConnection) -> QueryResult<Vec<PlanetEntity>> {
    use crate::persistence::schema::planets::dsl::*;

    planets.load(conn)
}

pub fn get(id: i32, conn: &PgConnection) -> QueryResult<PlanetEntity> {
    planets::table
        .find(id)
        .get_result(conn)
}

pub fn get_details(planet_id: i32, conn: &PgConnection) -> QueryResult<DetailsEntity> {
    details::table
        .filter(details::planet_id.eq(planet_id))
        .first(conn)
}

pub fn create(new_planet: NewPlanetEntity, mut new_details_entity: NewDetailsEntity, conn: &PgConnection) -> QueryResult<PlanetEntity> {
    use crate::persistence::schema::{planets::dsl::*, details::dsl::*};

    let result: QueryResult<PlanetEntity> = diesel::insert_into(planets)
        .values(new_planet)
        .get_result(conn);

    let new_planet_id = result.as_ref().ok().expect("Can't create planet").id;

    new_details_entity.planet_id = new_planet_id;

    diesel::insert_into(details)
        .values(new_details_entity)
        .execute(conn);

    result
}
