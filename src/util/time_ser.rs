//! Use the well-known [RFC3339 format] when serializing and deserializing an [`OffsetDateTime`].
//!
//! Use this module in combination with serde's [`#[with]`][with] attribute.
//!
//! [RFC3339 format]: https://tools.ietf.org/html/rfc3339#section-5.6
//! [with]: https://serde.rs/field-attrs.html#with

use core::fmt;
use core::marker::PhantomData;

use serde::{de, Deserializer, Serialize, Serializer};
use time::Format::Rfc3339;
use time::OffsetDateTime;

/// Serialize an [`OffsetDateTime`] using the well-known RFC3339 format.
pub fn serialize<S: Serializer>(
    datetime: &OffsetDateTime,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    datetime.format(Rfc3339).serialize(serializer)
}

/// Deserialize an [`OffsetDateTime`] from its RFC3339 representation.
pub fn deserialize<'a, D: Deserializer<'a>>(
    deserializer: D,
) -> Result<OffsetDateTime, D::Error> {
    deserializer.deserialize_any(Visitor(PhantomData))
}

pub(super) struct Visitor<T: ?Sized>(pub(super) PhantomData<T>);

impl<'a> de::Visitor<'a> for Visitor<OffsetDateTime> {
    type Value = OffsetDateTime;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("an `OffsetDateTime`")
    }

    fn visit_str<E: de::Error>(self, value: &str) -> Result<OffsetDateTime, E> {
        OffsetDateTime::parse(value, Rfc3339).map_err(E::custom)
    }
}
