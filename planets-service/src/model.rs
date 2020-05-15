use std::collections::HashMap;

use async_graphql::*;

#[derive(Clone)]
pub struct Planet {
    id: i32,
    name: &'static str,
    planet_type: Type,
    details: Details,
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
    async fn planet_type(&self) -> &Type {
        &self.planet_type
    }

    #[field(deprecation = "Now it is not in doubt. Do not use this field")]
    async fn is_rotating_around_sun(&self) -> bool {
        true
    }

    async fn details(&self) -> &Details {
        &self.details
    }
}

#[Enum]
enum Type {
    TerrestrialPlanet,
    GasGiant,
    IceGiant,
    DwarfPlanet,
}

#[Interface(
field(name = "mean_radius", type = "f32", context),
field(name = "mass", type = "f32", context),
)]
#[derive(Clone)]
enum Details {
    InhabitedPlanetDetails(InhabitedPlanetDetails),
    UninhabitedPlanetDetails(UninhabitedPlanetDetails),
}

#[derive(Clone)]
struct InhabitedPlanetDetails {
    mean_radius: f32,
    mass: f32,
    population: f32,
}

#[Object]
impl InhabitedPlanetDetails {
    async fn mean_radius(&self) -> f32 {
        self.mean_radius
    }

    async fn mass(&self) -> f32 {
        self.mass
    }

    #[field(desc = "in billions")]
    async fn population(&self) -> f32 {
        self.population
    }
}

#[derive(Clone)]
struct UninhabitedPlanetDetails {
    mean_radius: f32,
    mass: f32,
}

#[Object]
impl UninhabitedPlanetDetails {
    async fn mean_radius(&self) -> f32 {
        self.mean_radius
    }

    async fn mass(&self) -> f32 {
        self.mass
    }
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
            details: InhabitedPlanetDetails {
                mean_radius: 1.0,
                mass: 1.0,
                population: 7.0,
            }.into(),
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
