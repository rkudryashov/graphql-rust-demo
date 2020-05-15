use std::collections::HashMap;

use async_graphql::*;

#[derive(Clone)]
pub struct Planet {
    id: i32,
    name: &'static str,
    planet_type: Type,
}

#[Object]
impl Planet {
    async fn id(&self) -> i32 {
        self.id
    }

    async fn name(&self) -> &str {
        self.name
    }

    #[field(name = "type", desc = "From an astronomical point of view")]
    async fn planet_type(&self) -> Type {
        self.planet_type
    }

    #[field(deprecation = "Now it is not in doubt. Do not use this field")]
    async fn is_rotating_around_sun(&self) -> bool {
        true
    }
}

#[Enum]
enum Type {
    TerrestrialPlanet,
    GasGiant,
    IceGiant,
    DwarfPlanet,
}

pub struct Storage {
    planets: HashMap<&'static str, Planet>
}

impl Storage {
    pub fn new() -> Self {
        let earth = Planet {
            id: 1,
            name: "Earth",
            planet_type: Type::TerrestrialPlanet,
        };

        let mut planets = HashMap::new();

        planets.insert(earth.name, earth);

        Storage {
            planets
        }
    }

    pub fn planets(&self) -> Vec<Planet> {
        self.planets.values().cloned().collect()
    }
}
