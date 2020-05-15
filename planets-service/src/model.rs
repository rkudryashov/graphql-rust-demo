use std::collections::HashMap;

#[derive(Clone)]
pub struct Planet {
    id: i32,
    name: &'static str,
}

#[async_graphql::Object]
impl Planet {
    async fn id(&self) -> i32 {
        self.id
    }

    async fn name(&self) -> &str {
        self.name
    }
}

pub struct Storage {
    planets: HashMap<&'static str, Planet>
}

impl Storage {
    pub fn new() -> Self {
        let earth = Planet {
            id: 1,
            name: "Earth",
        };

        let mut planets = HashMap::new();

        planets.insert(earth.name, earth);

        Storage {
            planets
        }
    }

    pub fn planets(&self) -> Vec<Planet> {
        self.planets.values().cloned().collect()
    }
}
