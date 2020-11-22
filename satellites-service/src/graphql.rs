use std::str::FromStr;

use async_graphql::*;
use chrono::NaiveDate;
use strum_macros::EnumString;

use crate::get_conn_from_ctx;
use crate::persistence::model::SatelliteEntity;
use crate::persistence::repository;

pub type AppSchema = Schema<Query, EmptyMutation, EmptySubscription>;

pub struct Query;

#[Object]
impl Query {
    async fn get_satellites(&self, ctx: &Context<'_>) -> Vec<Satellite> {
        repository::get_all(&get_conn_from_ctx(ctx)).expect("Can't get satellites")
            .iter()
            .map(|e| { Satellite::from(e) })
            .collect()
    }

    async fn get_satellite(&self, ctx: &Context<'_>, id: ID) -> Option<Satellite> {
        let id = id.to_string().parse::<i32>().expect("Can't get id from String");
        repository::get(id, &get_conn_from_ctx(ctx)).ok()
            .map(|e| { Satellite::from(&e) })
    }

    #[graphql(entity)]
    async fn get_planet_by_id(&self, id: ID) -> Planet {
        Planet { id }
    }
}

#[derive(SimpleObject)]
struct Satellite {
    id: ID,
    name: String,
    life_exists: LifeExists,
    first_spacecraft_landing_date: Option<NaiveDate>,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Enum, EnumString)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum LifeExists {
    Yes,
    OpenQuestion,
    NoData,
}

struct Planet {
    id: ID
}

#[Object(extends)]
impl Planet {
    #[graphql(external)]
    async fn id(&self) -> &ID {
        &self.id
    }

    async fn satellites(&self, ctx: &Context<'_>) -> Vec<Satellite> {
        let id = self.id.to_string().parse::<i32>().expect("Can't get id from String");
        repository::get_by_planet_id(id, &get_conn_from_ctx(ctx)).expect("Can't get satellites of planet")
            .iter()
            .map(|e| { Satellite::from(e) })
            .collect()
    }
}

impl From<&SatelliteEntity> for Satellite {
    fn from(entity: &SatelliteEntity) -> Self {
        Satellite {
            id: entity.id.into(),
            name: entity.name.clone(),
            life_exists: LifeExists::from_str(entity.life_exists.as_str()).expect("Can't convert &str to LifeExists"),
            first_spacecraft_landing_date: entity.first_spacecraft_landing_date,
        }
    }
}
