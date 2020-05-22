use async_graphql::*;

use crate::model::{Planet, Satellite, Storage};

pub type TestSchema = Schema<Query, EmptyMutation, EmptySubscription>;

pub struct Query;

#[Object(extends)]
impl Query {
    async fn satellites(&self, ctx: &Context<'_>) -> Vec<Satellite> {
        ctx.data::<Storage>().satellites()
    }

    #[entity]
    async fn find_planet_by_id(&self, ctx: &Context<'_>, id: ID) -> Planet {
        Planet {
            id: id.clone(),
            satellites: ctx.data::<Storage>().satellites_by_planet_id(&id),
        }
    }
}
