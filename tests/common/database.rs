#![allow(dead_code)]

use labrinth::database::redis::RedisPool;
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::time::Duration;
use url::Url;

// The dummy test database adds a fair bit of 'dummy' data to test with.
// Some constants are used to refer to that data, and are described here.
// The rest can be accessed in the TestEnvironment 'dummy' field.

// The user IDs are as follows:
pub const ADMIN_USER_ID: &str = "1";
pub const MOD_USER_ID: &str = "2";
pub const USER_USER_ID: &str = "3"; // This is the 'main' user ID, and is used for most tests.
pub const FRIEND_USER_ID: &str = "4"; // This is exactly the same as USER_USER_ID, but could be used for testing friend-only endpoints (ie: teams, etc)
pub const ENEMY_USER_ID: &str = "5"; // This is exactly the same as USER_USER_ID, but could be used for testing friend-only endpoints (ie: teams, etc)

pub const ADMIN_USER_ID_PARSED: i64 = 1;
pub const MOD_USER_ID_PARSED: i64 = 2;
pub const USER_USER_ID_PARSED: i64 = 3;
pub const FRIEND_USER_ID_PARSED: i64 = 4;
pub const ENEMY_USER_ID_PARSED: i64 = 5;

// These are full-scoped PATs- as if the user was logged in (including illegal scopes).
pub const ADMIN_USER_PAT: &str = "mrp_patadmin";
pub const MOD_USER_PAT: &str = "mrp_patmoderator";
pub const USER_USER_PAT: &str = "mrp_patuser";
pub const FRIEND_USER_PAT: &str = "mrp_patfriend";
pub const ENEMY_USER_PAT: &str = "mrp_patenemy";

pub struct TemporaryDatabase {
    pub pool: PgPool,
    pub redis_pool: RedisPool,
    pub database_name: String,
}

impl TemporaryDatabase {
    // Creates a temporary database like sqlx::test does
    // 1. Logs into the main database
    // 2. Creates a new randomly generated database
    // 3. Runs migrations on the new database
    // 4. (Optionally, by using create_with_dummy) adds dummy data to the database
    // If a db is created with create_with_dummy, it must be cleaned up with cleanup.
    // This means that dbs will only 'remain' if a test fails (for examination of the db), and will be cleaned up otherwise.
    pub async fn create() -> Self {
        let temp_database_name = generate_random_database_name();
        println!("Creating temporary database: {}", &temp_database_name);

        let database_url = dotenvy::var("DATABASE_URL").expect("No database URL");
        let mut url = Url::parse(&database_url).expect("Invalid database URL");
        let pool = PgPool::connect(&database_url)
            .await
            .expect("Connection to database failed");

        // Create the temporary database
        let create_db_query = format!("CREATE DATABASE {}", &temp_database_name);

        sqlx::query(&create_db_query)
            .execute(&pool)
            .await
            .expect("Database creation failed");

        pool.close().await;

        // Modify the URL to switch to the temporary database
        url.set_path(&format!("/{}", &temp_database_name));
        let temp_db_url = url.to_string();

        let pool = PgPoolOptions::new()
            .min_connections(0)
            .max_connections(4)
            .max_lifetime(Some(Duration::from_secs(60 * 60)))
            .connect(&temp_db_url)
            .await
            .expect("Connection to temporary database failed");

        // Performs migrations
        let migrations = sqlx::migrate!("./migrations");
        migrations.run(&pool).await.expect("Migrations failed");

        // Gets new Redis pool
        let redis_pool = RedisPool::new(Some(temp_database_name.clone()));

        Self {
            pool,
            database_name: temp_database_name,
            redis_pool,
        }
    }

    // Deletes the temporary database
    // If a temporary db is created, it must be cleaned up with cleanup.
    // This means that dbs will only 'remain' if a test fails (for examination of the db), and will be cleaned up otherwise.
    pub async fn cleanup(mut self) {
        let database_url = dotenvy::var("DATABASE_URL").expect("No database URL");
        self.pool.close().await;

        self.pool = PgPool::connect(&database_url)
            .await
            .expect("Connection to main database failed");

        // Forcibly terminate all existing connections to this version of the temporary database
        // We are done and deleting it, so we don't need them anymore
        let terminate_query = format!(
            "SELECT pg_terminate_backend(pg_stat_activity.pid) FROM pg_stat_activity WHERE datname = '{}' AND pid <> pg_backend_pid()",
            &self.database_name
        );
        sqlx::query(&terminate_query)
            .execute(&self.pool)
            .await
            .unwrap();

        // Execute the deletion query asynchronously
        let drop_db_query = format!("DROP DATABASE IF EXISTS {}", &self.database_name);
        sqlx::query(&drop_db_query)
            .execute(&self.pool)
            .await
            .expect("Database deletion failed");
    }
}

fn generate_random_database_name() -> String {
    // Generate a random database name here
    // You can use your logic to create a unique name
    // For example, you can use a random string as you did before
    // or append a timestamp, etc.

    // We will use a random string starting with "labrinth_tests_db_"
    // and append a 6-digit number to it.
    let mut database_name = String::from("labrinth_tests_db_");
    database_name.push_str(&rand::random::<u64>().to_string()[..6]);
    database_name
}
