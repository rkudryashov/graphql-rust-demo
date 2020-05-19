use std::collections::HashMap;

use async_graphql::*;
use num_bigint::*;
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal_macros::dec;
use serde::Serialize;

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
field(name = "mean_radius", type = "&BigDecimal", context),
field(name = "mass", type = "&BigInt", context),
)]
#[derive(Clone)]
enum Details {
    InhabitedPlanetDetails(InhabitedPlanetDetails),
    UninhabitedPlanetDetails(UninhabitedPlanetDetails),
}

#[derive(Clone)]
struct InhabitedPlanetDetails {
    mean_radius: BigDecimal,
    mass: BigInt,
    population: BigDecimal,
}

#[Object]
impl InhabitedPlanetDetails {
    async fn mean_radius(&self) -> &BigDecimal {
        &self.mean_radius
    }

    async fn mass(&self) -> &BigInt {
        &self.mass
    }

    #[field(desc = "in billions")]
    async fn population(&self) -> &BigDecimal {
        &self.population
    }
}

#[derive(Clone)]
struct UninhabitedPlanetDetails {
    mean_radius: BigDecimal,
    mass: BigInt,
}

#[Object]
impl UninhabitedPlanetDetails {
    async fn mean_radius(&self) -> &BigDecimal {
        &self.mean_radius
    }

    async fn mass(&self) -> &BigInt {
        &self.mass
    }
}

#[derive(Clone, Serialize)]
pub struct BigInt(pub num_bigint::BigInt);

#[derive(Clone, Serialize)]
pub struct BigDecimal(pub Decimal);

#[Scalar]
impl ScalarType for BigInt {
    fn parse(value: Value) -> InputValueResult<Self> {
        unimplemented!()
    }

    fn to_json(&self) -> Result<serde_json::Value> {
        Ok(serde_json::to_value(&self.0.to_f64()).expect("Can't get json from BigInt"))
    }
}

#[Scalar]
impl ScalarType for BigDecimal {
    fn parse(value: Value) -> InputValueResult<Self> {
        unimplemented!()
    }

    fn to_json(&self) -> Result<serde_json::Value> {
        Ok(serde_json::to_value(&self.0).expect("Can't get json from Decimal"))
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
                mean_radius: BigDecimal(dec!(6371.0)),
                mass: BigInt(5.97e24_f64.to_bigint().expect("Can't get BigInt")),
                population: BigDecimal(dec!(7.53)),
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
