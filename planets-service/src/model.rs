use async_graphql::*;
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use serde::Serialize;

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
