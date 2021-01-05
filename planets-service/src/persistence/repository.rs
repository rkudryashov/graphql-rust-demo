use diesel::dsl::any;
use diesel::prelude::*;

use crate::persistence::model::{DetailsEntity, NewDetailsEntity, NewPlanetEntity, PlanetEntity};
use crate::persistence::schema::{details, planets};

pub fn get_all(conn: &PgConnection) -> QueryResult<Vec<PlanetEntity>> {
    use crate::persistence::schema::planets::dsl::*;

    planets.load(conn)
}

pub fn get(id: i32, conn: &PgConnection) -> QueryResult<PlanetEntity> {
    planets::table
        .find(id)
        .get_result(conn)
}

pub fn get_details(planet_ids: &[i32], conn: &PgConnection) -> QueryResult<Vec<DetailsEntity>> {
    details::table
        .filter(details::planet_id.eq(any(planet_ids)))
        .load::<DetailsEntity>(conn)
}

pub fn create(new_planet: NewPlanetEntity, mut new_details_entity: NewDetailsEntity, conn: &PgConnection) -> QueryResult<PlanetEntity> {
    use crate::persistence::schema::{planets::dsl::*, details::dsl::*};

    let created_planet: PlanetEntity = diesel::insert_into(planets)
        .values(new_planet)
        .get_result(conn)?;

    new_details_entity.planet_id = created_planet.id;

    diesel::insert_into(details)
        .values(new_details_entity)
        .execute(conn);

    Ok(created_planet)
}
