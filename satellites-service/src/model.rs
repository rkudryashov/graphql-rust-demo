use std::collections::HashMap;

use async_graphql::*;
use chrono::prelude::*;
use serde::Serialize;

#[derive(Clone)]
pub struct Satellite {
    id: ID,
    name: &'static str,
    life_exists: LifeExists,
    first_spacecraft_landing_date: NaiveDate,
    planet_id: i32,
}

#[Object]
impl Satellite {
    async fn id(&self) -> &ID {
        &self.id
    }

    async fn name(&self) -> &str {
        self.name
    }

    async fn life_exists(&self) -> &LifeExists {
        &self.life_exists
    }

    async fn first_spacecraft_landing_date(&self) -> &NaiveDate {
        &self.first_spacecraft_landing_date
    }
}

#[Enum]
enum LifeExists {
    Yes,
    OpenQuestion,
    NoData,
}

#[derive(Clone, Serialize)]
struct Date(NaiveDate);

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
        ctx.data::<Storage>().satellites_by_planet_id(&self.id)
    }
}

pub struct Storage {
    satellites: HashMap<&'static str, Satellite>
}

impl Storage {
    pub fn new() -> Self {
        let moon = Satellite {
            id: "1".into(),
            name: "Moon",
            life_exists: LifeExists::OpenQuestion,
            first_spacecraft_landing_date: NaiveDate::from_ymd(1959, 9, 13),
            planet_id: 3,
        };

        let mut satellites = HashMap::new();

        satellites.insert(moon.name, moon);

        Storage {
            satellites
        }
    }

    pub fn satellites(&self) -> Vec<Satellite> {
        self.satellites.values().cloned().collect()
    }

    pub fn satellites_by_planet_id(&self, planet_id: &ID) -> Vec<Satellite> {
        self.satellites().iter().cloned().filter(|s| {
            let planet_id: i32 = planet_id.parse::<i32>().expect("Can't parse String to i32");
            s.planet_id == planet_id
        }).collect()
    }
}
