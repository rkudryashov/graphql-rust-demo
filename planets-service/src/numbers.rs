use num_bigint::BigInt;
use rust_decimal::Decimal;
use serde::Serialize;

#[derive(Clone, Serialize)]
pub struct CustomBigInt(pub BigInt);

#[derive(Clone, Serialize)]
pub struct CustomDecimal(pub Decimal);
