//! Shared types for Modrinth App analytics: the events the app records, the
//! batch format it sends them in, and the extractor the ingest service uses
//! to validate a received batch.

pub mod enums;
pub mod event;
pub mod http;
pub mod iso3166_alpha2;
pub mod normal;
