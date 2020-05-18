use std::collections::HashMap;

use async_graphql::*;
use num_bigint::*;
use rust_decimal_macros::dec;

use crate::numbers::{CustomBigInt, CustomDecimal};

#[derive(Clone)]
pub struct Planet {
    pub id: ID,
    name: &'static str,
    planet_type: PlanetType,
    details: Details,
}

#[Object]
impl Planet {
    async fn id(&self) -> &ID {
        &self.id
    }

    async fn name(&self) -> &str {
        self.name
    }

    #[field(name = "type", desc = "From an astronomical point of view")]
    async fn planet_type(&self) -> &PlanetType {
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
enum PlanetType {
    TerrestrialPlanet,
    GasGiant,
    IceGiant,
    DwarfPlanet,
}

#[Interface(
field(name = "mean_radius", type = "&CustomDecimal", context),
field(name = "mass", type = "&CustomBigInt", context),
)]
#[derive(Clone)]
enum Details {
    InhabitedPlanetDetails(InhabitedPlanetDetails),
    UninhabitedPlanetDetails(UninhabitedPlanetDetails),
}

#[derive(Clone)]
struct InhabitedPlanetDetails {
    mean_radius: CustomDecimal,
    mass: CustomBigInt,
    population: CustomDecimal,
}

#[Object]
impl InhabitedPlanetDetails {
    async fn mean_radius(&self) -> &CustomDecimal {
        &self.mean_radius
    }

    async fn mass(&self) -> &CustomBigInt {
        &self.mass
    }

    #[field(desc = "in billions")]
    async fn population(&self) -> &CustomDecimal {
        &self.population
    }
}

#[derive(Clone)]
struct UninhabitedPlanetDetails {
    mean_radius: CustomDecimal,
    mass: CustomBigInt,
}

#[Object]
impl UninhabitedPlanetDetails {
    async fn mean_radius(&self) -> &CustomDecimal {
        &self.mean_radius
    }

    async fn mass(&self) -> &CustomBigInt {
        &self.mass
    }
}

pub struct Storage {
    planets: HashMap<&'static str, Planet>
}

impl Storage {
    pub fn new() -> Self {
        let earth = Planet {
            id: "1".into(),
            name: "Earth",
            planet_type: PlanetType::TerrestrialPlanet,
            details: InhabitedPlanetDetails {
                mean_radius: CustomDecimal(dec!(6371.0)),
                mass: CustomBigInt(5.97e24_f64.to_bigint().expect("Can't get BigInt")),
                population: CustomDecimal(dec!(7.53)),
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
