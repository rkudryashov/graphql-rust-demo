use diesel::prelude::*;

use crate::persistence::model::SatelliteEntity;
use crate::persistence::schema::satellites;

pub fn all(connection: &PgConnection) -> QueryResult<Vec<SatelliteEntity>> {
    satellites::table
        .load(connection)
}

pub fn get_by_planet_id(id_of_planet: i32, connection: &PgConnection) -> QueryResult<Vec<SatelliteEntity>> {
    use crate::persistence::schema::satellites::dsl::*;

    satellites
        .filter(planet_id.eq(id_of_planet))
        .load(connection)
}
