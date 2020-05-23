use std::str::FromStr;

use async_graphql::*;
use chrono::prelude::*;
use strum_macros::EnumString;

use crate::persistence::connection::PgPool;
use crate::persistence::model::SatelliteEntity;
use crate::persistence::repository;

pub type TestSchema = Schema<Query, EmptyMutation, EmptySubscription>;

pub struct Query;

#[Object(extends)]
impl Query {
    async fn satellites(&self, ctx: &Context<'_>) -> Vec<Satellite> {
        let conn = ctx.data::<PgPool>().get().expect("Can't get DB connection");

        let satellite_entities = repository::all(&conn).expect("Can't get satellites");

        satellite_entities.iter()
            .map(|e| { convert(e) })
            .collect()
    }

    #[entity]
    async fn find_planet_by_id(&self, ctx: &Context<'_>, id: ID) -> Planet {
        let conn = ctx.data::<PgPool>().get().expect("Can't get DB connection");

        let int_id = id.to_string().parse::<i32>().expect("Can't get id from String");
        let satellite_entities = repository::get_by_planet_id(int_id, &conn).expect("Can't get satellites of planet");

        let satellites = satellite_entities.iter()
            .map(|e| { convert(e) })
            .collect();

        Planet {
            id: id.clone(),
            satellites,
        }
    }
}

#[SimpleObject]
#[derive(Clone)]
pub struct Satellite {
    pub id: ID,
    pub name: String,
    pub life_exists: LifeExists,
    pub first_spacecraft_landing_date: Option<NaiveDate>,
    pub planet_id: i32,
}

#[Enum]
#[derive(EnumString)]
pub enum LifeExists {
    Yes,
    OpenQuestion,
    NoData,
}

#[derive(Clone)]
pub struct Planet {
    pub id: ID,
    pub satellites: Vec<Satellite>,
}

#[Object(extends)]
impl Planet {
    #[field(external)]
    async fn id(&self) -> &ID {
        &self.id
    }

    async fn satellites(&self, ctx: &Context<'_>) -> Vec<Satellite> {
        let conn = ctx.data::<PgPool>().get().expect("Can't get DB connection");

        let int_id = self.id.to_string().parse::<i32>().expect("Can't get id from String");
        let satellite_entities = repository::get_by_planet_id(int_id, &conn).expect("Can't get satellites of planet");

        let satellites = satellite_entities.iter()
            .map(|e| { convert(e) })
            .collect();

        satellites
    }
}

// todo from/into trait
fn convert(satellite_entity: &SatelliteEntity) -> Satellite {
    Satellite {
        id: satellite_entity.id.into(),
        name: satellite_entity.name.clone(),
        life_exists: LifeExists::from_str(satellite_entity.life_exists.as_str()).expect("Can't convert &str to LifeExists"),
        first_spacecraft_landing_date: satellite_entity.first_spacecraft_landing_date,
        planet_id: satellite_entity.planet_id,
    }
}
