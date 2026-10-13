use std::fmt;

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use sha2::Digest;
use toasty::{
    schema::{Field, Load},
    stmt::{Assign, Assignment, Expr, IntoExpr, List, Path, Type, Value},
};

use crate::util::de_error;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Sha256(pub [u8; 32]);

impl Sha256 {
    #[must_use]
    pub fn from_digest(data: impl AsRef<[u8]>) -> Self {
        let sha256 = sha2::Sha256::digest(data);
        let sha256 = sha256.as_array::<32>().expect("should be 32 bytes long");
        Self(*sha256)
    }
}

impl fmt::Debug for Sha256 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&hex::encode(self.0))
    }
}

impl fmt::Display for Sha256 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&hex::encode(self.0))
    }
}

impl Serialize for Sha256 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&hex::encode(self.0))
    }
}

impl<'de> Deserialize<'de> for Sha256 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        let mut bytes = [0; 32];
        hex::decode_to_slice(value, &mut bytes).map_err(de_error::<D>)?;
        Ok(Self(bytes))
    }
}

impl Load for Sha256 {
    type Output = Self;

    fn ty() -> Type {
        Type::String
    }

    fn load(value: Value) -> toasty::Result<Self> {
        match value {
            Value::String(value) => {
                let mut bytes = [0; 32];
                hex::decode_to_slice(value, &mut bytes).map_err(|error| {
                    toasty::Error::from_args(format_args!(
                        "invalid SHA-256 digest: {error}"
                    ))
                })?;
                Ok(Self(bytes))
            }
            _ => Err(toasty::Error::type_conversion(value, "Sha256")),
        }
    }

    fn reload(target: &mut Self, value: Value) -> toasty::Result<()> {
        *target = Self::load(value)?;
        Ok(())
    }
}

impl Field for Sha256 {
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

impl IntoExpr<Sha256> for Sha256 {
    fn into_expr(self) -> Expr<Sha256> {
        Expr::from_untyped(Value::String(hex::encode(self.0)))
    }

    fn by_ref(&self) -> Expr<Sha256> {
        Expr::from_untyped(Value::String(hex::encode(self.0)))
    }
}

impl Assign<Sha256> for Sha256 {
    fn into_assignment(self) -> Assignment<Sha256> {
        toasty::stmt::set(self.into_expr())
    }
}

impl toasty::codegen_support::index::IndexableField for Sha256 {}
