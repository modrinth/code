macro_rules! impl_redis_serde {
    (
        $type:ty,
        human = $human:ident,
        binary = $binary:ident,
        schema $(,)?
    ) => {
        $crate::database::redis::serde::impl_redis_serde!(
            $type,
            human = $human,
            binary = $binary,
        );

        impl ::utoipa::PartialSchema for $type {
            fn schema()
            -> ::utoipa::openapi::RefOr<::utoipa::openapi::schema::Schema> {
                <$human as ::utoipa::PartialSchema>::schema()
            }
        }

        impl ::utoipa::ToSchema for $type {
            fn schemas(
                schemas: &mut ::std::vec::Vec<(
                    ::std::string::String,
                    ::utoipa::openapi::RefOr<::utoipa::openapi::schema::Schema>,
                )>,
            ) {
                <$human as ::utoipa::ToSchema>::schemas(schemas);
            }
        }
    };
    (
        $type:ty,
        human = $human:ident,
        binary = $binary:ident $(,)?
    ) => {
        impl ::serde::Serialize for $type {
            fn serialize<S>(
                &self,
                serializer: S,
            ) -> ::std::result::Result<S::Ok, S::Error>
            where
                S: ::serde::Serializer,
            {
                if serializer.is_human_readable() {
                    $human::serialize(self, serializer)
                } else {
                    $binary::serialize(self, serializer)
                }
            }
        }

        impl<'de> ::serde::Deserialize<'de> for $type {
            fn deserialize<D>(
                deserializer: D,
            ) -> ::std::result::Result<Self, D::Error>
            where
                D: ::serde::Deserializer<'de>,
            {
                if deserializer.is_human_readable() {
                    $human::deserialize(deserializer)
                } else {
                    $binary::deserialize(deserializer)
                }
            }
        }
    };
}

pub(crate) use impl_redis_serde;
