use std::collections::HashMap;

use async_graphql::*;

#[derive(Clone)]
pub struct Satellite {
    id: ID,
    name: &'static str,
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
            planet_id: 1,
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
