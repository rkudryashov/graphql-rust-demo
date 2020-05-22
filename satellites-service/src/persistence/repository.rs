use diesel::prelude::*;

use crate::persistence::model::SatelliteEntity;
use crate::schema::satellites;

pub fn all(connection: &PgConnection) -> QueryResult<Vec<SatelliteEntity>> {
    satellites::table
        .load(connection)
}

pub fn get_by_planet_id(planet_id: i32, connection: &PgConnection) -> QueryResult<Vec<SatelliteEntity>> {
    use crate::schema::satellites::dsl::*;

    satellites
        .filter(planet_id.eq(planet_id))
        .load(connection)
}
