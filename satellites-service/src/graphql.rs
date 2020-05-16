use async_graphql::*;

use crate::model::{Satellite, Storage};

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn satellites(
        &self,
        ctx: &Context<'_>,
    ) -> Vec<Satellite> {
        ctx.data::<Storage>().satellites()
    }
}
