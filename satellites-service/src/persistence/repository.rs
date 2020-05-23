use diesel::prelude::*;

use crate::persistence::model::SatelliteEntity;
use crate::persistence::schema::satellites;

pub fn all(conn: &PgConnection) -> QueryResult<Vec<SatelliteEntity>> {
    satellites::table
        .load(conn)
}

pub fn get(id: i32, conn: &PgConnection) -> QueryResult<SatelliteEntity> {
    satellites::table
        .find(id)
        .get_result(conn)
}

pub fn get_by_planet_id(id_of_planet: i32, conn: &PgConnection) -> QueryResult<Vec<SatelliteEntity>> {
    use crate::persistence::schema::satellites::dsl::*;

    satellites
        .filter(planet_id.eq(id_of_planet))
        .load(conn)
}
