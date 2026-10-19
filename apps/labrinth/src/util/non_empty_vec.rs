use derive_more::{Display, Error};
use serde::{Deserialize, Deserializer, Serialize, de::Error as _};
use std::ops::Deref;

#[derive(Debug, Display, Error)]
#[display("must have at least one item")]
pub struct Empty;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct NonEmptyVec<T>(Vec<T>);

impl<T> Deref for NonEmptyVec<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> IntoIterator for NonEmptyVec<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<T> TryFrom<Vec<T>> for NonEmptyVec<T> {
    type Error = Empty;

    fn try_from(value: Vec<T>) -> Result<Self, Self::Error> {
        if value.is_empty() {
            Err(Empty)
        } else {
            Ok(Self(value))
        }
    }
}

impl<'de, T> Deserialize<'de> for NonEmptyVec<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Vec::deserialize(deserializer)
            .and_then(|value| Self::try_from(value).map_err(D::Error::custom))
    }
}

impl<T> utoipa::__dev::ComposeSchema for NonEmptyVec<T>
where
    T: utoipa::__dev::ComposeSchema,
{
    fn compose(
        schemas: Vec<utoipa::openapi::RefOr<utoipa::openapi::schema::Schema>>,
    ) -> utoipa::openapi::RefOr<utoipa::openapi::schema::Schema> {
        let item = schemas
            .first()
            .cloned()
            .unwrap_or_else(|| T::compose(schemas));
        utoipa::openapi::ArrayBuilder::new()
            .items(item)
            .min_items(Some(1))
            .into()
    }
}

impl<T> utoipa::ToSchema for NonEmptyVec<T>
where
    T: utoipa::ToSchema + utoipa::__dev::ComposeSchema,
{
    fn schemas(
        schemas: &mut Vec<(
            String,
            utoipa::openapi::RefOr<utoipa::openapi::schema::Schema>,
        )>,
    ) {
        T::schemas(schemas);
    }
}
