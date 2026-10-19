use eyre::{Result, WrapErr};

pub struct MinecraftMetaDb(pub toasty::Db);

impl MinecraftMetaDb {
	pub async fn connect() -> Result<Self> {
		let database_url = &crate::env::ENV.MINECRAFT_META_DATABASE_URL;

		let db = toasty::Db::builder()
			.models(toasty::models!(minecraft_meta_model::*))
			.connect(database_url.as_str())
			.await
			.wrap_err("connecting to Minecraft metadata database")?;
		Ok(Self(db))
	}
}

pub mod advisory_lock;
pub mod models;
mod postgres_database;
pub mod redis;
pub use models::DBImage;
pub use models::DBProject;
pub use models::DBVersion;
pub use postgres_database::{
    Acquire, Executor, MIGRATOR, PgPool, PgTransaction, ReadOnlyPgPool,
    check_for_migrations, connect_all, register_and_set_metrics,
};
