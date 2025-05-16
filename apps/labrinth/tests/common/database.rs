#![allow(dead_code)]

use labrinth::{database::redis::RedisPool, search};
use sqlx::{PgPool, postgres::PgPoolOptions};
use std::time::Duration;
use url::Url;

use crate::common::{dummy_data, environment::TestEnvironment};

use super::{api_v3::ApiV3, dummy_data::DUMMY_DATA_UPDATE};

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
pub const ADMIN_USER_PAT: Option<&str> = Some("mrp_patadmin");
pub const MOD_USER_PAT: Option<&str> = Some("mrp_patmoderator");
pub const USER_USER_PAT: Option<&str> = Some("mrp_patuser");
pub const FRIEND_USER_PAT: Option<&str> = Some("mrp_patfriend");
pub const ENEMY_USER_PAT: Option<&str> = Some("mrp_patenemy");

const TEMPLATE_DATABASE_NAME: &str = "labrinth_tests_template";

#[derive(Clone)]
pub struct TemporaryDatabase {
    pub pool: PgPool,
    pub redis_pool: RedisPool,
    pub search_config: labrinth::search::SearchConfig,
    pub database_name: String,
}

impl TemporaryDatabase {
    // Creates a temporary database like sqlx::test does (panics)
    // 1. Logs into the main database
    // 2. Creates a new randomly generated database
    // 3. Runs migrations on the new database
    // 4. (Optionally, by using create_with_dummy) adds dummy data to the database
    // If a db is created with create_with_dummy, it must be cleaned up with cleanup.
    // This means that dbs will only 'remain' if a test fails (for examination of the db), and will be cleaned up otherwise.
    pub async fn create(max_connections: Option<u32>) -> Self {
        let temp_database_name = generate_random_name("labrinth_tests_db_");
        println!("Creating temporary database: {}", &temp_database_name);

        let database_url =
            dotenvy::var("DATABASE_URL").expect("No database URL");

        // Create the temporary (and template datbase, if needed)
        Self::create_temporary(&database_url, &temp_database_name).await;

        // Pool to the temporary database
        let mut temporary_url =
            Url::parse(&database_url).expect("Invalid database URL");

        temporary_url.set_path(&format!("/{}", &temp_database_name));
        let temp_db_url = temporary_url.to_string();

        let pool = PgPoolOptions::new()
            .min_connections(0)
            .max_connections(max_connections.unwrap_or(4))
            .max_lifetime(Some(Duration::from_secs(60)))
            .connect(&temp_db_url)
            .await
            .expect("Connection to temporary database failed");

        println!("Running migrations on temporary database");

        // Performs migrations
        let migrations = sqlx::migrate!("./migrations");
        migrations.run(&pool).await.expect("Migrations failed");

        println!("Migrations complete");

        // Gets new Redis pool
        let redis_pool = RedisPool::new(Some(temp_database_name.clone()));

        // Create new meilisearch config
        let search_config =
            search::SearchConfig::new(Some(temp_database_name.clone()));
        Self {
            pool,
            database_name: temp_database_name,
            redis_pool,
            search_config,
        }
    }

    // Creates a template and temporary databse (panics)
    // 1. Waits to obtain a pg lock on the main database
    // 2. Creates a new template database called 'TEMPLATE_DATABASE_NAME', if needed
    // 3. Switches to the template database
    // 4. Runs migrations on the new database (for most tests, this should not take time)
    // 5. Creates dummy data on the new db
    // 6. Creates a temporary database at 'temp_database_name' from the template
    // 7. Drops lock and all created connections in the function
    async fn create_temporary(database_url: &str, temp_database_name: &str) {
        let main_pool = PgPool::connect(database_url)
            .await
            .expect("Connection to database failed");

        loop {
            // Try to acquire an advisory lock
            let lock_acquired: bool =
                sqlx::query_scalar("SELECT pg_try_advisory_lock(1)")
                    .fetch_one(&main_pool)
                    .await
                    .unwrap();

            if lock_acquired {
                // Create the db template if it doesn't exist
                // Check if template_db already exists
                let db_exists: Option<i32> = sqlx::query_scalar(&format!(
                    "SELECT 1 FROM pg_database WHERE datname = '{TEMPLATE_DATABASE_NAME}'"
                ))
                .fetch_optional(&main_pool)
                .await
                .unwrap();
                if db_exists.is_none() {
                    create_template_database(&main_pool).await;
                }

                // Switch to template
                let url =
                    dotenvy::var("DATABASE_URL").expect("No database URL");
                let mut template_url =
                    Url::parse(&url).expect("Invalid database URL");
                template_url.set_path(&format!("/{TEMPLATE_DATABASE_NAME}"));

                let pool = PgPool::connect(template_url.as_str())
                    .await
                    .expect("Connection to database failed");

                // Check if dummy data exists- a fake 'dummy_data' table is created if it does
                let mut dummy_data_exists: bool = sqlx::query_scalar(
                    "SELECT to_regclass('dummy_data') IS NOT NULL",
                )
                .fetch_one(&pool)
                .await
                .unwrap();
                if dummy_data_exists {
                    // Check if the dummy data needs to be updated
                    let dummy_data_update = sqlx::query_scalar::<_, i64>(
                        "SELECT update_id FROM dummy_data",
                    )
                    .fetch_optional(&pool)
                    .await
                    .unwrap();
                    let needs_update = dummy_data_update
                        .is_none_or(|d| d != DUMMY_DATA_UPDATE);
                    if needs_update {
                        println!(
                            "Dummy data updated, so template DB tables will be dropped and re-created"
                        );
                        // Drop all tables in the database so they can be re-created and later filled with updated dummy data
                        sqlx::query("DROP SCHEMA public CASCADE;")
                            .execute(&pool)
                            .await
                            .unwrap();
                        sqlx::query("CREATE SCHEMA public;")
                            .execute(&pool)
                            .await
                            .unwrap();
                        dummy_data_exists = false;
                    }
                }

                // Run migrations on the template
                let migrations = sqlx::migrate!("./migrations");
                migrations.run(&pool).await.expect("Migrations failed");

                if !dummy_data_exists {
                    // Add dummy data
                    let name = generate_random_name("test_template_");
                    let db = TemporaryDatabase {
                        pool: pool.clone(),
                        database_name: TEMPLATE_DATABASE_NAME.to_string(),
                        redis_pool: RedisPool::new(Some(name.clone())),
                        search_config: search::SearchConfig::new(Some(name)),
                    };
                    let setup_api =
                        TestEnvironment::<ApiV3>::build_setup_api(&db).await;
                    dummy_data::add_dummy_data(&setup_api, db.clone()).await;
                    db.pool.close().await;
                }
                pool.close().await;
                drop(pool);

                // Create the temporary database from the template
                let create_db_query = format!(
                    "CREATE DATABASE {} TEMPLATE {}",
                    &temp_database_name, TEMPLATE_DATABASE_NAME
                );

                sqlx::query(&create_db_query)
                    .execute(&main_pool)
                    .await
                    .expect("Database creation failed");

                // Release the advisory lock
                sqlx::query("SELECT pg_advisory_unlock(1)")
                    .execute(&main_pool)
                    .await
                    .unwrap();

                main_pool.close().await;
                break;
            }
            // Wait for the lock to be released
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }
    }

    // Deletes the temporary database (panics)
    // If a temporary db is created, it must be cleaned up with cleanup.
    // This means that dbs will only 'remain' if a test fails (for examination of the db), and will be cleaned up otherwise.
    pub async fn cleanup(mut self) {
        let database_url =
            dotenvy::var("DATABASE_URL").expect("No database URL");
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
        let drop_db_query =
            format!("DROP DATABASE IF EXISTS {}", &self.database_name);
        sqlx::query(&drop_db_query)
            .execute(&self.pool)
            .await
            .expect("Database deletion failed");
    }
}

async fn create_template_database(pool: &sqlx::Pool<sqlx::Postgres>) {
    let create_db_query = format!("CREATE DATABASE {TEMPLATE_DATABASE_NAME}");
    sqlx::query(&create_db_query)
        .execute(pool)
        .await
        .expect("Database creation failed");
}

// Appends a random 8-digit number to the end of the str
pub fn generate_random_name(str: &str) -> String {
    let mut str = String::from(str);
    str.push_str(&rand::random::<u64>().to_string()[..8]);
    str
}
