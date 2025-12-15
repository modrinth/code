macro_rules! display_as_serialize {
    ($T:ty) => {
        const _: () = {
            use std::fmt;

            impl fmt::Display for $T {
                fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    let value = serde_json::to_value(self).map_err(|_| fmt::Error)?;
                    let value = value.as_str().ok_or(fmt::Error)?;
                    write!(f, "{value}")
                }
            }
        };
    };
}
pub(crate) use display_as_serialize;
