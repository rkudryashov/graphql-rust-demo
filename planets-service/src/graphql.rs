use async_graphql::*;

use crate::model::{Planet, Storage};

pub struct Query;

pub type TestSchema = Schema<Query, EmptyMutation, EmptySubscription>;

#[Object]
impl Query {
    async fn planets(&self, ctx: &Context<'_>) -> Vec<Planet> {
        ctx.data::<Storage>().planets()
    }

    #[entity]
    async fn find_planet_by_id(&self, ctx: &Context<'_>, id: ID) -> Option<Planet> {
        ctx.data::<Storage>().planets().iter().cloned()
            .find(|p| {
                p.id == id
            })
    }
}
