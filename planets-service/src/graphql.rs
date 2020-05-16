use async_graphql::*;
use rust_decimal::prelude::ToPrimitive;

use crate::model::{Planet, Storage};
use crate::numbers::{CustomBigInt, CustomDecimal};

pub struct QueryRoot;

pub type TestSchema = Schema<QueryRoot, EmptyMutation, EmptySubscription>;

#[Object]
impl QueryRoot {
    async fn planets(
        &self,
        ctx: &Context<'_>,
    ) -> Vec<Planet> {
        ctx.data::<Storage>().planets()
    }
}

#[Scalar]
impl ScalarType for CustomBigInt {
    fn type_name() -> &'static str {
        "BigInt"
    }

    fn parse(value: Value) -> InputValueResult<Self> {
        unimplemented!()
    }

    fn to_json(&self) -> Result<serde_json::Value> {
        Ok(serde_json::to_value(&self.0.to_f64()).expect("Can't get json from BigInt"))
    }
}


#[Scalar]
impl ScalarType for CustomDecimal {
    fn type_name() -> &'static str {
        "Decimal"
    }

    fn parse(value: Value) -> InputValueResult<Self> {
        unimplemented!()
    }

    fn to_json(&self) -> Result<serde_json::Value> {
        Ok(serde_json::to_value(&self.0).expect("Can't get json from Decimal"))
    }
}
