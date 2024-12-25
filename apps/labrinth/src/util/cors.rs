use ntex_cors::{Cors, CorsFactory};

pub fn default_cors<Err>() -> CorsFactory<Err> {
    Cors::default()
}
