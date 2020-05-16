use std::collections::HashMap;

use async_graphql::*;

#[derive(Clone)]
pub struct Satellite {
    id: ID,
    name: &'static str,
}

#[Object]
impl Satellite {
    async fn id(&self) -> &ID {
        &self.id
    }

    async fn name(&self) -> &str {
        self.name
    }
}

pub struct Storage {
    satellites: HashMap<&'static str, Satellite>
}

impl Storage {
    pub fn new() -> Self {
        let moon = Satellite {
            id: "1".into(),
            name: "Moon",
        };

        let mut satellites = HashMap::new();

        satellites.insert(moon.name, moon);

        Storage {
            satellites
        }
    }

    pub fn satellites(&self) -> Vec<Satellite> {
        self.satellites.values().cloned().collect()
    }
}
