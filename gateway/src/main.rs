use anyhow::Result;
use dotenv::dotenv;

mod jwt_validation;

fn main() -> Result<()> {
    dotenv().ok();
    apollo_router::main()
}
