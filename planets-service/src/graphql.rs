use std::ops::Mul;
use std::str::FromStr;

use async_graphql::*;
use num_bigint::*;
use rust_decimal::prelude::ToPrimitive;
use serde::Serialize;
use strum_macros::{Display, EnumString};

use crate::persistence::connection::PgPool;
use crate::persistence::model::{DetailsEntity, NewDetailsEntity, NewPlanetEntity, PlanetEntity};
use crate::persistence::repository;

pub type TestSchema = Schema<Query, Mutation, EmptySubscription>;

pub struct Query;

#[Object]
impl Query {
    async fn planets(&self, ctx: &Context<'_>) -> Vec<Planet> {
        let conn = ctx.data::<PgPool>().get().expect("Can't get DB connection");

        let planet_entities = repository::all(&conn).expect("Can't get planets");

        planet_entities.iter()
            .map(|(p, d)| { convert(p, d) })
            .collect()
    }

    async fn planet(&self, ctx: &Context<'_>, id: ID) -> Option<Planet> {
        self.find_planet_by_id_internal(ctx, id).await.unwrap()
    }

    #[entity]
    async fn find_planet_by_id(&self, ctx: &Context<'_>, id: ID) -> Option<Planet> {
        self.find_planet_by_id_internal(ctx, id).await.unwrap()
    }

    async fn find_planet_by_id_internal(&self, ctx: &Context<'_>, id: ID) -> Option<Planet> {
        let conn = ctx.data::<PgPool>().get().expect("Can't get DB connection");

        let id = id.to_string().parse::<i32>().expect("Can't get id from String");
        let maybe_planet_and_details = repository::get(id, &conn).ok();

        maybe_planet_and_details.map(|(planet_entity, details_entity)| {
            convert(&planet_entity, &details_entity)
        })
    }
}

pub struct Mutation;

#[Object]
impl Mutation {
    async fn create_planet(&self, ctx: &Context<'_>, name: String, planet_type: PlanetType, details: DetailsInput) -> ID {
        fn get_new_planet_mass(number: f32, ten_power: usize) -> bigdecimal::BigDecimal {
            let some = bigdecimal::BigDecimal::from(number);
            some.mul(num::pow(bigdecimal::BigDecimal::from(10), ten_power))
        }

        let conn = ctx.data::<PgPool>().get().expect("Can't get DB connection");

        let new_planet = NewPlanetEntity {
            name,
            planet_type: planet_type.to_string(),
        };

        let new_planet_details = NewDetailsEntity {
            mean_radius: details.mean_radius.0,
            mass: get_new_planet_mass(details.mass.number, details.mass.ten_power as usize),
            population: details.population.map(|v| { v.0 }),
            planet_id: 0,
        };

        let create_planet_result = repository::create(new_planet, new_planet_details, &conn);

        create_planet_result.expect("Can't create new planet").id.into()
    }
}

#[derive(Clone)]
pub struct Planet {
    pub id: ID,
    pub name: String,
    pub planet_type: PlanetType,
    pub details: Details,
}

#[Object]
impl Planet {
    async fn id(&self) -> &ID {
        &self.id
    }

    async fn name(&self) -> &String {
        &self.name
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
#[derive(Display, EnumString)]
pub enum PlanetType {
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
pub enum Details {
    InhabitedPlanetDetails(InhabitedPlanetDetails),
    UninhabitedPlanetDetails(UninhabitedPlanetDetails),
}

#[derive(Clone)]
pub struct InhabitedPlanetDetails {
    pub mean_radius: BigDecimal,
    pub mass: BigInt,
    pub population: BigDecimal,
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
pub struct UninhabitedPlanetDetails {
    pub mean_radius: BigDecimal,
    pub mass: BigInt,
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

#[Scalar]
impl ScalarType for BigInt {
    fn parse(value: Value) -> InputValueResult<Self> {
        unimplemented!()
    }

    fn to_value(&self) -> Value {
        // convert to float to represent as number with mantissa and exponent
        // todo test other options
        Value::Float(self.0.to_f64().expect("Can't get f64"))
    }
}

#[derive(Clone, Serialize)]
pub struct BigDecimal(pub bigdecimal::BigDecimal);

#[Scalar]
impl ScalarType for BigDecimal {
    fn parse(value: Value) -> InputValueResult<Self> {
        match value {
            Value::String(s) => {
                let parsed_value = bigdecimal::BigDecimal::from_str(s.as_str())?;
                Ok(BigDecimal(parsed_value))
            }
            _ => Err(InputValueError::ExpectedType(value)),
        }
    }

    fn to_value(&self) -> Value {
        // todo test other options to get rid of quotes
        Value::String(self.0.to_string())
    }
}

#[InputObject]
pub struct DetailsInput {
    pub mean_radius: BigDecimal,
    pub mass: MassInput,
    pub population: Option<BigDecimal>,
}

#[InputObject]
pub struct MassInput {
    pub number: f32,
    pub ten_power: i8,
}

// todo from/into trait
fn convert(planet_entity: &PlanetEntity, details_entity: &DetailsEntity) -> Planet {
    let details: Details = if details_entity.population.is_some() {
        InhabitedPlanetDetails {
            mean_radius: BigDecimal(details_entity.mean_radius.clone()),
            mass: BigInt(details_entity.mass.to_bigint().clone().expect("Can't get mass")),
            population: BigDecimal(details_entity.population.as_ref().expect("Can't get population").clone()),
        }.into()
    } else {
        UninhabitedPlanetDetails {
            mean_radius: BigDecimal(details_entity.mean_radius.clone()),
            mass: BigInt(details_entity.mass.to_bigint().clone().expect("Can't get mass")),
        }.into()
    };

    Planet {
        id: planet_entity.id.into(),
        name: planet_entity.name.clone(),
        planet_type: PlanetType::from_str(planet_entity.planet_type.as_str()).expect("Can't convert &str to PlanetType"),
        details,
    }
}
