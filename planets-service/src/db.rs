use bigdecimal::BigDecimal;

use crate::schema::{details, planets};

#[derive(Identifiable, Queryable)]
#[table_name = "planets"]
pub struct PlanetEntity {
    pub id: i32,
    pub name: String,
    #[column_name = "type"]
    pub planet_type: String,
}

#[derive(Identifiable, Queryable, Associations)]
#[table_name = "details"]
#[belongs_to(PlanetEntity, foreign_key = "planet_id")]
// todo store in 2 different tables
pub struct DetailsEntity {
    pub id: i32,
    pub mean_radius: BigDecimal,
    pub mass: BigDecimal,
    pub population: Option<BigDecimal>,
    pub planet_id: i32,
}

#[derive(Insertable)]
#[table_name = "planets"]
pub struct NewPlanetEntity {
    pub name: String,
}
