use std::collections::HashMap;
use std::ops::Mul;
use std::str::FromStr;
use std::sync::Arc;

use async_graphql::*;
use bigdecimal::{BigDecimal, ToPrimitive};
use dataloader::BatchFn;
use dataloader::non_cached::Loader;
use futures::Stream;
use num_bigint::{BigInt, ToBigInt};
use strum_macros::{Display, EnumString};

use async_trait::async_trait;

use crate::get_conn_from_ctx;
use crate::persistence::connection::PgPool;
use crate::persistence::model::{DetailsEntity, NewDetailsEntity, NewPlanetEntity, PlanetEntity};
use crate::persistence::repository;

pub type AppSchema = Schema<Query, Mutation, Subscription>;

pub struct Query;

#[Object]
impl Query {
    async fn planets(&self, ctx: &Context<'_>) -> Vec<Planet> {
        let planet_entities = repository::all(&get_conn_from_ctx(ctx)).expect("Can't get planets");

        planet_entities.iter()
            .map(|p| { Planet::from(p) })
            .collect()
    }

    async fn planet(&self, ctx: &Context<'_>, id: ID) -> Option<Planet> {
        find_planet_by_id_internal(ctx, id)
    }

    #[entity]
    async fn find_planet_by_id(&self, ctx: &Context<'_>, id: ID) -> Option<Planet> {
        find_planet_by_id_internal(ctx, id)
    }
}

fn find_planet_by_id_internal(ctx: &Context<'_>, id: ID) -> Option<Planet> {
    let id = id.to_string().parse::<i32>().expect("Can't get id from String");
    let maybe_planet = repository::get(id, &get_conn_from_ctx(ctx)).ok();

    maybe_planet.map(|p| { Planet::from(&p) })
}

pub struct Mutation;

#[Object]
impl Mutation {
    async fn create_planet(&self, ctx: &Context<'_>, name: String, planet_type: PlanetType, details: DetailsInput) -> ID {
        fn get_new_planet_mass(number: f32, ten_power: i8) -> BigDecimal {
            let some = BigDecimal::from(number);
            some.mul(num::pow(BigDecimal::from(10), ten_power as usize))
        }

        let new_planet = NewPlanetEntity {
            name,
            planet_type: planet_type.to_string(),
        };

        let new_planet_details = NewDetailsEntity {
            mean_radius: details.mean_radius.0,
            mass: get_new_planet_mass(details.mass.number, details.mass.ten_power),
            population: details.population.map(|wrapper| { wrapper.0 }),
            planet_id: 0,
        };

        let created_planet_entity = repository::create(new_planet, new_planet_details, &get_conn_from_ctx(ctx)).expect("Can't create planet");

        SimpleBroker::publish(Planet::from(&created_planet_entity));

        created_planet_entity.id.into()
    }
}

pub struct Subscription;

#[Subscription]
impl Subscription {
    async fn latest_planet(&self) -> impl Stream<Item=Planet> {
        SimpleBroker::<Planet>::subscribe()
    }
}

#[derive(Clone)]
struct Planet {
    id: ID,
    name: String,
    planet_type: PlanetType,
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

    async fn details(&self, ctx: &Context<'_>) -> Details {
        let loader = ctx.data::<Loader<ID, Details, DetailsBatchLoader>>().expect("Can't get loader");
        loader.load(self.id.clone()).await
    }
}

#[Enum]
#[derive(Display, EnumString)]
enum PlanetType {
    TerrestrialPlanet,
    GasGiant,
    IceGiant,
    DwarfPlanet,
}

#[Interface(
    field(name = "mean_radius", type = "&CustomBigDecimal"),
    field(name = "mass", type = "&CustomBigInt"),
)]
#[derive(Clone)]
pub enum Details {
    InhabitedPlanetDetails(InhabitedPlanetDetails),
    UninhabitedPlanetDetails(UninhabitedPlanetDetails),
}

#[SimpleObject]
#[derive(Clone)]
pub struct InhabitedPlanetDetails {
    mean_radius: CustomBigDecimal,
    mass: CustomBigInt,
    #[field(desc = "in billions")]
    population: CustomBigDecimal,
}

#[SimpleObject]
#[derive(Clone)]
pub struct UninhabitedPlanetDetails {
    mean_radius: CustomBigDecimal,
    mass: CustomBigInt,
}

#[derive(Clone)]
struct CustomBigInt(BigInt);

#[Scalar(name = "BigInt")]
impl ScalarType for CustomBigInt {
    fn parse(value: Value) -> InputValueResult<Self> {
        unimplemented!()
    }

    fn to_value(&self) -> Value {
        // convert to float to represent as number with mantissa and exponent
        // todo test other options
        Value::Float(self.0.to_f64().expect("Can't get f64"))
    }
}

#[derive(Clone)]
struct CustomBigDecimal(BigDecimal);

#[Scalar(name = "BigDecimal")]
impl ScalarType for CustomBigDecimal {
    fn parse(value: Value) -> InputValueResult<Self> {
        match value {
            Value::String(s) => {
                let parsed_value = BigDecimal::from_str(s.as_str())?;
                Ok(CustomBigDecimal(parsed_value))
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
struct DetailsInput {
    mean_radius: CustomBigDecimal,
    mass: MassInput,
    population: Option<CustomBigDecimal>,
}

#[InputObject]
struct MassInput {
    number: f32,
    ten_power: i8,
}

impl From<&PlanetEntity> for Planet {
    fn from(entity: &PlanetEntity) -> Self {
        Planet {
            id: entity.id.into(),
            name: entity.name.clone(),
            planet_type: PlanetType::from_str(entity.planet_type.as_str()).expect("Can't convert &str to PlanetType"),
        }
    }
}

impl From<&DetailsEntity> for Details {
    fn from(entity: &DetailsEntity) -> Self {
        if entity.population.is_some() {
            InhabitedPlanetDetails {
                mean_radius: CustomBigDecimal(entity.mean_radius.clone()),
                mass: CustomBigInt(entity.mass.to_bigint().clone().expect("Can't get mass")),
                population: CustomBigDecimal(entity.population.as_ref().expect("Can't get population").clone()),
            }.into()
        } else {
            UninhabitedPlanetDetails {
                mean_radius: CustomBigDecimal(entity.mean_radius.clone()),
                mass: CustomBigInt(entity.mass.to_bigint().clone().expect("Can't get mass")),
            }.into()
        }
    }
}

pub struct DetailsBatchLoader {
    pub pool: Arc<PgPool>
}

#[async_trait]
impl BatchFn<ID, Details> for DetailsBatchLoader {
    async fn load(&self, keys: &[ID]) -> HashMap<ID, Details> {
        keys.iter().map(|planet_id| {
            let conn = self.pool.get().expect("Can't get DB connection");

            let planet_id_int = planet_id.to_string().parse::<i32>().expect("Can't convert id");
            let details_entity = repository::get_details(planet_id_int, &conn).expect("Can't get details for a planet");

            (planet_id.clone(), Details::from(&details_entity))
        }).collect::<HashMap<_, _>>()
    }
}
