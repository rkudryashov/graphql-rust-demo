use chrono::NaiveDate;

use crate::persistence::schema::satellites;

#[derive(Identifiable, Queryable)]
#[table_name = "satellites"]
pub struct SatelliteEntity {
    pub id: i32,
    pub name: String,
    pub life_exists: String,
    pub first_spacecraft_landing_date: Option<NaiveDate>,
    pub planet_id: i32,
}
