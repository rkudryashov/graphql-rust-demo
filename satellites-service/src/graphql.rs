use std::str::FromStr;

use async_graphql::*;
use chrono::prelude::*;
use strum_macros::EnumString;

use crate::persistence::connection::PgPool;
use crate::persistence::model::SatelliteEntity;
use crate::persistence::repository;
use crate::RequestContext;
use crate::utils::decode_token;

pub type AppSchema = Schema<Query, EmptyMutation, EmptySubscription>;

pub struct Query;

#[Object(extends)]
impl Query {
    async fn satellites(&self, ctx: &Context<'_>) -> Vec<Satellite> {
        let conn = ctx.data::<PgPool>().get().expect("Can't get DB connection");

        let satellite_entities = repository::all(&conn).expect("Can't get satellites");

        satellite_entities.iter()
            .map(|e| { Satellite::from(e) })
            .collect()
    }

    async fn satellite(&self, ctx: &Context<'_>, id: ID) -> Option<Satellite> {
        let conn = ctx.data::<PgPool>().get().expect("Can't get DB connection");

        let id = id.to_string().parse::<i32>().expect("Can't get id from String");
        repository::get(id, &conn).ok()
            .map(|e| { Satellite::from(&e) })
    }

    #[entity]
    async fn get_planet_by_id(&self, id: ID) -> Planet {
        Planet { id: id.clone() }
    }
}

#[derive(Clone)]
struct Satellite {
    id: ID,
    name: String,
    life_exists: LifeExists,
    first_spacecraft_landing_date: Option<NaiveDate>,
    planet_id: i32,
}

#[Object]
impl Satellite {
    async fn id(&self) -> &ID {
        &self.id
    }

    async fn name(&self) -> &str {
        &self.name
    }

    async fn life_exists(&self, ctx: &Context<'_>) -> &LifeExists {
        let maybe_token = &ctx.data::<RequestContext>().token;
        if let Some(token) = maybe_token {
            let token_data = decode_token(token);
            if token_data.claims.role == "admin" {
                return &self.life_exists;
            }
        }

        panic!("life_exists can only be accessed by authenticated user with `admin` role")
    }

    async fn first_spacecraft_landing_date(&self) -> &Option<NaiveDate> {
        &self.first_spacecraft_landing_date
    }
}

#[Enum]
#[derive(EnumString)]
enum LifeExists {
    Yes,
    OpenQuestion,
    NoData,
}

#[derive(Clone)]
struct Planet {
    id: ID
}

#[Object(extends)]
impl Planet {
    #[field(external)]
    async fn id(&self) -> &ID {
        &self.id
    }

    async fn satellites(&self, ctx: &Context<'_>) -> Vec<Satellite> {
        let conn = ctx.data::<PgPool>().get().expect("Can't get DB connection");

        let id = self.id.to_string().parse::<i32>().expect("Can't get id from String");
        let satellite_entities = repository::get_by_planet_id(id, &conn).expect("Can't get satellites of planet");

        let satellites = satellite_entities.iter()
            .map(|e| { Satellite::from(e) })
            .collect();

        satellites
    }
}

impl From<&SatelliteEntity> for Satellite {
    fn from(entity: &SatelliteEntity) -> Self {
        Satellite {
            id: entity.id.into(),
            name: entity.name.clone(),
            life_exists: LifeExists::from_str(entity.life_exists.as_str()).expect("Can't convert &str to LifeExists"),
            first_spacecraft_landing_date: entity.first_spacecraft_landing_date,
            planet_id: entity.planet_id,
        }
    }
}
