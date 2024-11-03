#[macro_export]
macro_rules! bitflags_serde_impl {
    ($type:ident, $int_type:ident) => {
        impl serde::Serialize for $type {
            fn serialize<S: serde::Serializer>(
                &self,
                serializer: S,
            ) -> Result<S::Ok, S::Error> {
                serializer.serialize_i64(self.bits() as i64)
            }
        }

        impl<'de> serde::Deserialize<'de> for $type {
            fn deserialize<D: serde::Deserializer<'de>>(
                deserializer: D,
            ) -> Result<Self, D::Error> {
                let v: i64 = Deserialize::deserialize(deserializer)?;

                Ok($type::from_bits_truncate(v as $int_type))
            }
        }
    };
}
