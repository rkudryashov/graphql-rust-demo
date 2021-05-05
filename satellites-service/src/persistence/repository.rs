use diesel::prelude::*;

use crate::persistence::model::SatelliteEntity;
use crate::persistence::schema::satellites;

pub fn get_all(conn: &PgConnection) -> QueryResult<Vec<SatelliteEntity>> {
    use crate::persistence::schema::satellites::dsl::*;

    satellites.load(conn)
}

pub fn get(id: i32, conn: &PgConnection) -> QueryResult<SatelliteEntity> {
    satellites::table.find(id).get_result(conn)
}

pub fn get_by_planet_id(planet_id: i32, conn: &PgConnection) -> QueryResult<Vec<SatelliteEntity>> {
    satellites::table
        .filter(satellites::planet_id.eq(planet_id))
        .load(conn)
}
