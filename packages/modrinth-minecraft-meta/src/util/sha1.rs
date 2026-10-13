use std::fmt;

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use sha2::Digest;
use toasty::{
    schema::{Field, Load},
    stmt::{Assign, Assignment, Expr, IntoExpr, List, Path, Type, Value},
};

use crate::util::de_error;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Sha1(pub [u8; 20]);

impl Sha1 {
    #[must_use]
    pub fn from_digest(data: impl AsRef<[u8]>) -> Self {
        let sha1 = sha1::Sha1::digest(data);
        let sha1 = sha1.as_array::<20>().expect("should be 20 bytes long");
        Self(*sha1)
    }
}

impl fmt::Debug for Sha1 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&hex::encode(self.0))
    }
}

impl fmt::Display for Sha1 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&hex::encode(self.0))
    }
}

impl Serialize for Sha1 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&hex::encode(self.0))
    }
}

impl<'de> Deserialize<'de> for Sha1 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        let mut bytes = [0; 20];
        hex::decode_to_slice(value, &mut bytes).map_err(de_error::<D>)?;
        Ok(Self(bytes))
    }
}

impl Load for Sha1 {
    type Output = Self;

    fn ty() -> Type {
        Type::String
    }

    fn load(value: Value) -> toasty::Result<Self> {
        match value {
            Value::String(value) => {
                let mut bytes = [0; 20];
                hex::decode_to_slice(value, &mut bytes).map_err(|error| {
                    toasty::Error::from_args(format_args!(
                        "invalid SHA-1 digest: {error}"
                    ))
                })?;
                Ok(Self(bytes))
            }
            _ => Err(toasty::Error::type_conversion(value, "Sha1")),
        }
    }

    fn reload(target: &mut Self, value: Value) -> toasty::Result<()> {
        *target = Self::load(value)?;
        Ok(())
    }
}

impl Field for Sha1 {
    type ExprTarget = Self;
    type Path<Origin> = Path<Origin, Self>;
    type ListPath<Origin> = Path<Origin, List<Self>>;
    type Update<'a> = ();
    type Inner = Self;

    fn new_path<Origin>(path: Path<Origin, Self>) -> Self::Path<Origin> {
        path
    }

    fn new_list_path<Origin>(
        path: Path<Origin, List<Self>>,
    ) -> Self::ListPath<Origin> {
        path
    }

    fn new_update<'a>(
        _assignments: &'a mut toasty::codegen_support::core::stmt::Assignments,
        _projection: toasty::stmt::Projection,
    ) -> Self::Update<'a> {
    }

    fn key_constraint<Origin>(
        &self,
        target: Path<Origin, Self::Inner>,
    ) -> Expr<bool> {
        target.eq(self)
    }
}

impl IntoExpr<Sha1> for Sha1 {
    fn into_expr(self) -> Expr<Sha1> {
        Expr::from_untyped(Value::String(hex::encode(self.0)))
    }

    fn by_ref(&self) -> Expr<Sha1> {
        Expr::from_untyped(Value::String(hex::encode(self.0)))
    }
}

impl Assign<Sha1> for Sha1 {
    fn into_assignment(self) -> Assignment<Sha1> {
        toasty::stmt::set(self.into_expr())
    }
}

impl toasty::codegen_support::index::IndexableField for Sha1 {}
