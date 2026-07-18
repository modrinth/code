// for macro expansion
extern crate self as cached_projection;

use std::collections::{BTreeMap, HashMap};
use std::hash::Hash;
use std::sync::Arc;

pub use cached_projection_derive::CachedProjection;
use serde::de::DeserializeOwned;
use serde::ser::{SerializeMap, SerializeSeq};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

pub trait CachedProjection: Sized {
    type ProjectedType: Serialize + DeserializeOwned;

    type ProjectedTypeRef<'a>: Serialize
    where
        Self: 'a;

    fn project_ref(&self) -> Self::ProjectedTypeRef<'_>;

    fn into_projected(self) -> Self::ProjectedType;

    fn from_projected(projected: Self::ProjectedType) -> Self;
}

pub struct DeserializeAnyJsonWrapper<T>(pub T);

impl<T> Serialize for DeserializeAnyJsonWrapper<T>
where
    T: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serde_json::to_string(&self.0)
            .map_err(serde::ser::Error::custom)?
            .serialize(serializer)
    }
}

impl<'de, T> Deserialize<'de> for DeserializeAnyJsonWrapper<T>
where
    T: DeserializeOwned,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        serde_json::from_str(&value)
            .map(Self)
            .map_err(serde::de::Error::custom)
    }
}

pub struct DeserializeAnyJsonWrapperRef<'a, T: ?Sized>(pub &'a T);

impl<T> Serialize for DeserializeAnyJsonWrapperRef<'_, T>
where
    T: Serialize + ?Sized,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serde_json::to_string(self.0)
            .map_err(serde::ser::Error::custom)?
            .serialize(serializer)
    }
}

macro_rules! identity_projection {
	($($ty:ty),* $(,)?) => {
		$(
			impl CachedProjection for $ty {
				type ProjectedType = Self;
				type ProjectedTypeRef<'a> = &'a Self;

				fn project_ref(&self) -> Self::ProjectedTypeRef<'_> {
					self
				}

				fn into_projected(self) -> Self::ProjectedType {
					self
				}

				fn from_projected(projected: Self::ProjectedType) -> Self {
					projected
				}
			}
		)*
	};
}

identity_projection!(
    (),
    bool,
    char,
    String,
    i8,
    i16,
    i32,
    i64,
    i128,
    isize,
    u8,
    u16,
    u32,
    u64,
    u128,
    usize,
    f32,
    f64,
);

impl<T> CachedProjection for Option<T>
where
    T: CachedProjection,
{
    type ProjectedType = Option<T::ProjectedType>;
    type ProjectedTypeRef<'a>
        = Option<T::ProjectedTypeRef<'a>>
    where
        Self: 'a;

    fn project_ref(&self) -> Self::ProjectedTypeRef<'_> {
        self.as_ref().map(CachedProjection::project_ref)
    }

    fn into_projected(self) -> Self::ProjectedType {
        self.map(CachedProjection::into_projected)
    }

    fn from_projected(projected: Self::ProjectedType) -> Self {
        projected.map(CachedProjection::from_projected)
    }
}

pub struct ProjectedSliceRef<'a, T>(&'a [T]);

impl<T> Serialize for ProjectedSliceRef<'_, T>
where
    T: CachedProjection,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut sequence = serializer.serialize_seq(Some(self.0.len()))?;
        for value in self.0 {
            sequence.serialize_element(&value.project_ref())?;
        }
        sequence.end()
    }
}

impl<T> CachedProjection for Vec<T>
where
    T: CachedProjection,
{
    type ProjectedType = Vec<T::ProjectedType>;
    type ProjectedTypeRef<'a>
        = ProjectedSliceRef<'a, T>
    where
        Self: 'a;

    fn project_ref(&self) -> Self::ProjectedTypeRef<'_> {
        ProjectedSliceRef(self)
    }

    fn into_projected(self) -> Self::ProjectedType {
        self.into_iter()
            .map(CachedProjection::into_projected)
            .collect()
    }

    fn from_projected(projected: Self::ProjectedType) -> Self {
        projected
            .into_iter()
            .map(CachedProjection::from_projected)
            .collect()
    }
}

impl<T> CachedProjection for Box<T>
where
    T: CachedProjection,
{
    type ProjectedType = T::ProjectedType;
    type ProjectedTypeRef<'a>
        = T::ProjectedTypeRef<'a>
    where
        Self: 'a;

    fn project_ref(&self) -> Self::ProjectedTypeRef<'_> {
        self.as_ref().project_ref()
    }

    fn into_projected(self) -> Self::ProjectedType {
        (*self).into_projected()
    }

    fn from_projected(projected: Self::ProjectedType) -> Self {
        Box::new(T::from_projected(projected))
    }
}

impl<T> CachedProjection for Arc<T>
where
    T: CachedProjection + Clone,
{
    type ProjectedType = T::ProjectedType;
    type ProjectedTypeRef<'a>
        = T::ProjectedTypeRef<'a>
    where
        Self: 'a;

    fn project_ref(&self) -> Self::ProjectedTypeRef<'_> {
        self.as_ref().project_ref()
    }

    fn into_projected(self) -> Self::ProjectedType {
        Arc::unwrap_or_clone(self).into_projected()
    }

    fn from_projected(projected: Self::ProjectedType) -> Self {
        Arc::new(T::from_projected(projected))
    }
}

pub struct ProjectedHashMapRef<'a, K, V>(&'a HashMap<K, V>);

impl<K, V> Serialize for ProjectedHashMapRef<'_, K, V>
where
    K: Serialize,
    V: CachedProjection,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.0.len()))?;
        for (key, value) in self.0 {
            map.serialize_entry(key, &value.project_ref())?;
        }
        map.end()
    }
}

impl<K, V> CachedProjection for HashMap<K, V>
where
    K: Serialize + DeserializeOwned + Eq + Hash,
    V: CachedProjection,
{
    type ProjectedType = HashMap<K, V::ProjectedType>;
    type ProjectedTypeRef<'a>
        = ProjectedHashMapRef<'a, K, V>
    where
        Self: 'a;

    fn project_ref(&self) -> Self::ProjectedTypeRef<'_> {
        ProjectedHashMapRef(self)
    }

    fn into_projected(self) -> Self::ProjectedType {
        self.into_iter()
            .map(|(key, value)| (key, value.into_projected()))
            .collect()
    }

    fn from_projected(projected: Self::ProjectedType) -> Self {
        projected
            .into_iter()
            .map(|(key, value)| (key, V::from_projected(value)))
            .collect()
    }
}

pub struct ProjectedBTreeMapRef<'a, K, V>(&'a BTreeMap<K, V>);

impl<K, V> Serialize for ProjectedBTreeMapRef<'_, K, V>
where
    K: Serialize,
    V: CachedProjection,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.0.len()))?;
        for (key, value) in self.0 {
            map.serialize_entry(key, &value.project_ref())?;
        }
        map.end()
    }
}

impl<K, V> CachedProjection for BTreeMap<K, V>
where
    K: Serialize + DeserializeOwned + Ord,
    V: CachedProjection,
{
    type ProjectedType = BTreeMap<K, V::ProjectedType>;
    type ProjectedTypeRef<'a>
        = ProjectedBTreeMapRef<'a, K, V>
    where
        Self: 'a;

    fn project_ref(&self) -> Self::ProjectedTypeRef<'_> {
        ProjectedBTreeMapRef(self)
    }

    fn into_projected(self) -> Self::ProjectedType {
        self.into_iter()
            .map(|(key, value)| (key, value.into_projected()))
            .collect()
    }

    fn from_projected(projected: Self::ProjectedType) -> Self {
        projected
            .into_iter()
            .map(|(key, value)| (key, V::from_projected(value)))
            .collect()
    }
}

impl<A, B> CachedProjection for (A, B)
where
    A: CachedProjection,
    B: CachedProjection,
{
    type ProjectedType = (A::ProjectedType, B::ProjectedType);
    type ProjectedTypeRef<'a>
        = (A::ProjectedTypeRef<'a>, B::ProjectedTypeRef<'a>)
    where
        Self: 'a;

    fn project_ref(&self) -> Self::ProjectedTypeRef<'_> {
        (self.0.project_ref(), self.1.project_ref())
    }

    fn into_projected(self) -> Self::ProjectedType {
        (self.0.into_projected(), self.1.into_projected())
    }

    fn from_projected(projected: Self::ProjectedType) -> Self {
        (
            A::from_projected(projected.0),
            B::from_projected(projected.1),
        )
    }
}

impl<A, B, C> CachedProjection for (A, B, C)
where
    A: CachedProjection,
    B: CachedProjection,
    C: CachedProjection,
{
    type ProjectedType = (A::ProjectedType, B::ProjectedType, C::ProjectedType);
    type ProjectedTypeRef<'a>
        = (
        A::ProjectedTypeRef<'a>,
        B::ProjectedTypeRef<'a>,
        C::ProjectedTypeRef<'a>,
    )
    where
        Self: 'a;

    fn project_ref(&self) -> Self::ProjectedTypeRef<'_> {
        (
            self.0.project_ref(),
            self.1.project_ref(),
            self.2.project_ref(),
        )
    }

    fn into_projected(self) -> Self::ProjectedType {
        (
            self.0.into_projected(),
            self.1.into_projected(),
            self.2.into_projected(),
        )
    }

    fn from_projected(projected: Self::ProjectedType) -> Self {
        (
            A::from_projected(projected.0),
            B::from_projected(projected.1),
            C::from_projected(projected.2),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::CachedProjection;
    use serde::{Deserialize, Serialize};

    #[derive(CachedProjection, Serialize, Deserialize)]
    struct Child {
        value: String,
    }

    #[derive(CachedProjection, Serialize, Deserialize)]
    struct Parent {
        #[cached_projection(nested)]
        child: Child,
        #[cached_projection(wrap)]
        metadata: serde_json::Value,
    }

    #[test]
    fn postcard_projection_round_trip() {
        let value = Parent {
            child: Child {
                value: "value".to_owned(),
            },
            metadata: serde_json::json!({ "key": [1, 2, 3] }),
        };

        let bytes = postcard::to_allocvec(&value.project_ref()).unwrap();
        let projected =
            postcard::from_bytes::<ParentProjectedType>(&bytes).unwrap();
        let _value = Parent::from_projected(projected);
    }
}
