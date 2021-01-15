use std::collections::HashMap;
use std::env;
use std::fmt;
use std::fmt::Formatter;
use std::fmt::LowerExp;
use std::iter::Iterator;
use std::str::FromStr;
use std::sync::{Arc, Mutex};

use async_graphql::*;
use async_graphql::dataloader::{DataLoader, Loader};
use async_graphql::guard::Guard;
use bigdecimal::{BigDecimal, ToPrimitive};
use futures::{Stream, StreamExt};
use rdkafka::{Message, producer::FutureProducer};
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};

use common_utils::Role;

use crate::get_conn_from_ctx;
use crate::kafka;
use crate::persistence::connection::PgPool;
use crate::persistence::model::{DetailsEntity, NewDetailsEntity, NewPlanetEntity, PlanetEntity};
use crate::persistence::repository;

pub type AppSchema = Schema<Query, Mutation, Subscription>;

pub struct Query;

#[Object]
impl Query {
    async fn get_planets(&self, ctx: &Context<'_>) -> Vec<Planet> {
        repository::get_all(&get_conn_from_ctx(ctx)).expect("Can't get planets")
            .iter()
            .map(|p| { Planet::from(p) })
            .collect()
    }

    async fn get_planet(&self, ctx: &Context<'_>, id: ID) -> Option<Planet> {
        find_planet_by_id_internal(ctx, id)
    }

    #[graphql(entity)]
    async fn find_planet_by_id(&self, ctx: &Context<'_>, id: ID) -> Option<Planet> {
        find_planet_by_id_internal(ctx, id)
    }
}

fn find_planet_by_id_internal(ctx: &Context<'_>, id: ID) -> Option<Planet> {
    let id = id.to_string().parse::<i32>().expect("Can't get id from String");
    repository::get(id, &get_conn_from_ctx(ctx)).ok()
        .map(|p| { Planet::from(&p) })
}

pub struct Mutation;

#[Object]
impl Mutation {
    #[graphql(guard(RoleGuard(role = "Role::Admin")))]
    async fn create_planet(&self, ctx: &Context<'_>, planet: PlanetInput) -> Result<Planet, Error> {
        let new_planet = NewPlanetEntity {
            name: planet.name,
            planet_type: planet.planet_type.to_string(),
        };

        let details = planet.details;
        let new_planet_details = NewDetailsEntity {
            mean_radius: details.mean_radius.0,
            mass: BigDecimal::from_str(&details.mass.0.to_string()).expect("Can't get BigDecimal from string"),
            population: details.population.map(|wrapper| { wrapper.0 }),
            planet_id: 0,
        };

        let created_planet_entity = repository::create(new_planet, new_planet_details, &get_conn_from_ctx(ctx))?;

        let producer = ctx.data::<FutureProducer>().expect("Can't get Kafka producer");
        let message = serde_json::to_string(&Planet::from(&created_planet_entity)).expect("Can't serialize a planet");
        kafka::send_message(producer, message).await;

        Ok(Planet::from(&created_planet_entity))
    }
}

pub struct Subscription;

#[Subscription]
impl Subscription {
    async fn latest_planet<'ctx>(&self, ctx: &'ctx Context<'_>) -> impl Stream<Item=Planet> + 'ctx {
        let kafka_consumer_counter = ctx.data::<Mutex<i32>>().expect("Can't get Kafka consumer counter");
        let consumer_group_id = kafka::get_kafka_consumer_group_id(kafka_consumer_counter);
        let consumer = kafka::create_consumer(consumer_group_id);

        async_stream::stream! {
            let mut stream = consumer.start();

            while let Some(value) = stream.next().await {
                yield match value {
                    Ok(message) => {
                        let payload = message.payload().expect("Kafka message should contain payload");
                        let message = String::from_utf8_lossy(payload).to_string();
                        serde_json::from_str(&message).expect("Can't deserialize a planet")
                    }
                    Err(e) => panic!("Error while Kafka message processing: {}", e)
                };
            }
        }
    }
}

#[derive(Serialize, Deserialize)]
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

    /// From an astronomical point of view
    #[graphql(name = "type")]
    async fn planet_type(&self) -> &PlanetType {
        &self.planet_type
    }

    #[graphql(deprecation = "Now it is not in doubt. Do not use this field")]
    async fn is_rotating_around_sun(&self) -> bool {
        true
    }

    async fn details(&self, ctx: &Context<'_>) -> Result<Details> {
        let data_loader = ctx.data::<DataLoader<DetailsLoader>>().expect("Can't get data loader");
        let planet_id = self.id.to_string().parse::<i32>().expect("Can't convert id");
        let details = data_loader.load_one(planet_id).await?;
        details.ok_or_else(|| "Not found".into())
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Serialize, Deserialize, Enum, Display, EnumString)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
enum PlanetType {
    TerrestrialPlanet,
    GasGiant,
    IceGiant,
    DwarfPlanet,
}

#[derive(Interface, Clone)]
#[graphql(
    field(name = "mean_radius", type = "&CustomBigDecimal"),
    field(name = "mass", type = "&CustomBigInt"),
)]
pub enum Details {
    InhabitedPlanetDetails(InhabitedPlanetDetails),
    UninhabitedPlanetDetails(UninhabitedPlanetDetails),
}

#[derive(SimpleObject, Clone)]
pub struct InhabitedPlanetDetails {
    mean_radius: CustomBigDecimal,
    mass: CustomBigInt,
    /// In billions
    population: CustomBigDecimal,
}

#[derive(SimpleObject, Clone)]
pub struct UninhabitedPlanetDetails {
    mean_radius: CustomBigDecimal,
    mass: CustomBigInt,
}

#[derive(Clone)]
pub struct CustomBigInt(BigDecimal);

#[Scalar(name = "BigInt")]
impl ScalarType for CustomBigInt {
    fn parse(value: Value) -> InputValueResult<Self> {
        match value {
            Value::String(s) => {
                let parsed_value = BigDecimal::from_str(&s)?;
                Ok(CustomBigInt(parsed_value))
            }
            _ => Err(InputValueError::expected_type(value)),
        }
    }

    fn to_value(&self) -> Value {
        Value::String(format!("{:e}", &self))
    }
}

impl LowerExp for CustomBigInt {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let val = &self.0.to_f64().expect("Can't convert BigDecimal");
        LowerExp::fmt(val, f)
    }
}

#[derive(Clone)]
pub struct CustomBigDecimal(BigDecimal);

#[Scalar(name = "BigDecimal")]
impl ScalarType for CustomBigDecimal {
    fn parse(value: Value) -> InputValueResult<Self> {
        match value {
            Value::String(s) => {
                let parsed_value = BigDecimal::from_str(&s)?;
                Ok(CustomBigDecimal(parsed_value))
            }
            _ => Err(InputValueError::expected_type(value)),
        }
    }

    fn to_value(&self) -> Value {
        Value::String(self.0.to_string())
    }
}

#[derive(InputObject)]
struct PlanetInput {
    name: String,
    #[graphql(name = "type")]
    planet_type: PlanetType,
    details: DetailsInput,
}

#[derive(InputObject)]
struct DetailsInput {
    /// In kilometers
    mean_radius: CustomBigDecimal,
    /// In kilograms. A number should be represented as, for example, `6.42e+23`
    mass: CustomBigInt,
    /// In billions
    population: Option<CustomBigDecimal>,
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
                mass: CustomBigInt(entity.mass.clone()),
                population: CustomBigDecimal(entity.population.as_ref().expect("Can't get population").clone()),
            }.into()
        } else {
            UninhabitedPlanetDetails {
                mean_radius: CustomBigDecimal(entity.mean_radius.clone()),
                mass: CustomBigInt(entity.mass.clone()),
            }.into()
        }
    }
}

pub struct DetailsLoader {
    pub pool: Arc<PgPool>
}

#[async_trait::async_trait]
impl Loader<i32> for DetailsLoader {
    type Value = Details;
    type Error = Error;

    async fn load(&self, keys: &[i32]) -> Result<HashMap<i32, Self::Value>, Self::Error> {
        let conn = self.pool.get().expect("Can't get DB connection");
        let details = repository::get_details(keys, &conn).expect("Can't get planets' details");

        Ok(details.iter()
            .map(|details_entity| (details_entity.planet_id, Details::from(details_entity)))
            .collect::<HashMap<_, _>>())
    }
}

struct RoleGuard {
    role: Role,
}

#[async_trait::async_trait]
impl Guard for RoleGuard {
    async fn check(&self, ctx: &Context<'_>) -> Result<()> {
        // TODO: auth disabling is needed for tests. try to reimplement when https://github.com/rust-lang/rust/issues/45599 will be resolved (using cfg(test))
        if let Ok(boolean) = env::var("DISABLE_AUTH") {
            let disable_auth = bool::from_str(boolean.as_str()).expect("Can't parse bool");
            if disable_auth {
                return Ok(());
            }
        };
        if ctx.data_opt::<Role>() == Some(&self.role) {
            Ok(())
        } else {
            Err("Forbidden".into())
        }
    }
}
