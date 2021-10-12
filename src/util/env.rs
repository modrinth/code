use std::str::FromStr;

pub fn parse_var<T: FromStr>(var: &'static str) -> Option<T> {
    dotenv::var(var).ok().and_then(|i| i.parse().ok())
}
pub fn parse_strings_from_var(var: &'static str) -> Option<Vec<String>> {
    dotenv::var(var)
        .ok()
        .and_then(|s| serde_json::from_str::<Vec<String>>(&s).ok())
}
