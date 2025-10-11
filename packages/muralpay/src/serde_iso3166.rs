use rust_iso3166::CountryCode;

pub fn serialize<S: serde::Serializer>(
    v: &CountryCode,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(v.alpha2)
}

pub fn deserialize<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<CountryCode, D::Error> {
    struct Visitor;

    impl serde::de::Visitor<'_> for Visitor {
        type Value = CountryCode;

        fn expecting(
            &self,
            formatter: &mut std::fmt::Formatter,
        ) -> std::fmt::Result {
            write!(formatter, "an ISO 3166 alpha-2 country code")
        }

        fn visit_str<E>(self, v: &str) -> std::result::Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            rust_iso3166::ALPHA2_MAP.get(v).copied().ok_or_else(|| {
                E::custom("invalid ISO 3166 alpha-2 country code")
            })
        }
    }

    deserializer.deserialize_str(Visitor)
}
