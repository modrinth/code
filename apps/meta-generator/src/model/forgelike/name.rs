use std::{str::FromStr, sync::LazyLock};

use anyhow::ensure;
use derive_more::Display;
use regex::Regex;
use serde::{Deserialize, Serialize};
use toasty::{
    schema::{Field, Load},
    stmt::{Assign, Assignment, Expr, IntoExpr, List, Path, Type, Value},
};

/// Full upstream version name identifying a Forge or NeoForge installer.
///
/// Forge uses `<minecraft version>-<loader version>`; modern NeoForge uses
/// its loader version alone.
#[derive(
    Debug, Display, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize,
)]
pub struct ForgelikeInstallerName(String);

impl ForgelikeInstallerName {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl FromStr for ForgelikeInstallerName {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        static PATTERN: LazyLock<Regex> = LazyLock::new(|| {
            Regex::new(r"\A[A-Za-z0-9_+\-]+(?:\.[A-Za-z0-9_+\-]+)*\z").unwrap()
        });
        ensure!(
            PATTERN.is_match(value),
            "invalid Forge-like installer name: {value:?}"
        );
        Ok(Self(value.to_owned()))
    }
}

impl<'de> Deserialize<'de> for ForgelikeInstallerName {
    fn deserialize<D: serde::Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Self, D::Error> {
        String::deserialize(deserializer)?
            .parse()
            .map_err(serde::de::Error::custom)
    }
}

impl Load for ForgelikeInstallerName {
    type Output = Self;
    fn ty() -> Type {
        Type::String
    }
    fn load(value: Value) -> toasty::Result<Self> {
        match value {
            Value::String(value) => value.parse().map_err(|error| {
                toasty::Error::from_args(format_args!("{error}"))
            }),
            _ => Err(toasty::Error::type_conversion(
                value,
                "ForgelikeInstallerName",
            )),
        }
    }
    fn reload(target: &mut Self, value: Value) -> toasty::Result<()> {
        *target = Self::load(value)?;
        Ok(())
    }
}
impl Field for ForgelikeInstallerName {
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

impl IntoExpr<ForgelikeInstallerName> for ForgelikeInstallerName {
    fn into_expr(self) -> Expr<ForgelikeInstallerName> {
        Expr::from_untyped(Value::String(self.0.clone()))
    }

    fn by_ref(&self) -> Expr<ForgelikeInstallerName> {
        Expr::from_untyped(Value::String(self.0.clone()))
    }
}

impl Assign<ForgelikeInstallerName> for ForgelikeInstallerName {
    fn into_assignment(self) -> Assignment<ForgelikeInstallerName> {
        toasty::stmt::set(self.into_expr())
    }
}

impl toasty::codegen_support::index::IndexableField for ForgelikeInstallerName {}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn installer_names_validate_parsing_json_and_database_loading() {
        for value in [
            "1.20.1-47.1.7",
            "21.4.31-beta",
            "26.1.0.0-alpha.1+snapshot-1",
        ] {
            let name: ForgelikeInstallerName = value.parse().unwrap();
            assert_eq!(name.as_str(), value);
            assert_eq!(serde_json::to_value(&name).unwrap(), json!(value));
            assert_eq!(
                serde_json::from_value::<ForgelikeInstallerName>(json!(value))
                    .unwrap(),
                name
            );
            assert_eq!(
                ForgelikeInstallerName::load(Value::String(value.into()))
                    .unwrap(),
                name
            );
        }
        for value in [
            "", ".", "..", "a/b", "a\\b", "a?b", "a b", "a\n", ".foo", "foo.",
            "foo..bar",
        ] {
            assert!(
                value.parse::<ForgelikeInstallerName>().is_err(),
                "{value:?}"
            );
            assert!(
                serde_json::from_value::<ForgelikeInstallerName>(json!(value))
                    .is_err()
            );
            assert!(
                ForgelikeInstallerName::load(Value::String(value.into()))
                    .is_err()
            );
        }
    }
}
