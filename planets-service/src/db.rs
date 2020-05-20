use crate::schema::planets;

#[derive(Queryable)]
pub struct Planet {
    pub id: i32,
    pub name: String,
}

#[derive(Insertable)]
#[table_name = "planets"]
pub struct NewPlanet {
    pub name: String,
}
