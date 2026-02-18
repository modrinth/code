use std::{convert::Infallible, str::FromStr};

pub fn parse_var<T: FromStr>(var: &str) -> Option<T> {
    dotenvy::var(var).ok().and_then(|i| i.parse().ok())
}
