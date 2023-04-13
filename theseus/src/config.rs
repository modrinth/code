//! Configuration structs

use lazy_static::lazy_static;

lazy_static! {
    pub static ref BINCODE_CONFIG: bincode::config::Configuration =
        bincode::config::standard()
            .with_little_endian()
            .with_no_limit();
}

pub const MODRINTH_API_URL: &str = "https://api.modrinth.com/v2/";

pub fn sled_config() -> sled::Config {
    sled::Config::default().use_compression(true)
}
