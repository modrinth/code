use {
    rust_iso3166::CountryCode,
    serde::{Deserialize, de::Error},
    std::borrow::Cow,
};

pub fn serialize<S: serde::Serializer>(v: &CountryCode, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(v.alpha2)
}

pub fn deserialize<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<CountryCode, D::Error> {
    <Cow<'_, str>>::deserialize(deserializer).and_then(|country_code| {
        rust_iso3166::ALPHA2_MAP
            .get(&country_code)
            .copied()
            .ok_or_else(|| D::Error::custom("invalid ISO 3166 alpha-2 country code"))
    })
}
