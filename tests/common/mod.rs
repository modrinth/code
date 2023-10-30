use labrinth::{check_env_vars, clickhouse};
use labrinth::{file_hosting, queue, LabrinthConfig};
use std::sync::Arc;

use self::database::TemporaryDatabase;

pub mod actix;
pub mod api_v2;
pub mod api_v3;
pub mod asserts;
pub mod database;
pub mod dummy_data;
pub mod environment;
pub mod pats;
pub mod permissions;
pub mod request_data;
pub mod scopes;

// Testing equivalent to 'setup' function, producing a LabrinthConfig
// If making a test, you should probably use environment::TestEnvironment::build() (which calls this)
pub async fn setup(db: &TemporaryDatabase) -> LabrinthConfig {
    println!("Setting up labrinth config");

    dotenvy::dotenv().ok();

    if check_env_vars() {
        println!("Some environment variables are missing!");
    }

    let pool = db.pool.clone();
    let redis_pool = db.redis_pool.clone();
    let file_host: Arc<dyn file_hosting::FileHost + Send + Sync> =
        Arc::new(file_hosting::MockHost::new());
    let mut clickhouse = clickhouse::init_client().await.unwrap();

    let maxmind_reader = Arc::new(queue::maxmind::MaxMindIndexer::new().await.unwrap());

    labrinth::app_setup(
        pool.clone(),
        redis_pool.clone(),
        &mut clickhouse,
        file_host.clone(),
        maxmind_reader.clone(),
    )
}

pub fn get_json_val_str(val: impl serde::Serialize) -> String {
    serde_json::to_value(val)
        .unwrap()
        .as_str()
        .unwrap()
        .to_string()
}
