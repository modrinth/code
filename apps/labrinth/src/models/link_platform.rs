use derive_more::Display;
use serde::{Deserialize, Serialize};
use strum::{EnumIter, EnumString};

#[derive(
    Debug,
    Display,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    Serialize,
    Deserialize,
    EnumIter,
    EnumString,
    utoipa::ToSchema,
)]
#[display(rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum LinkPlatform {
    Patreon,
    Bmac,
    Paypal,
    Github,
    Kofi,
    Other,
    Issues,
    Wiki,
    Discord,
    Source,
    Site,
    Store,
}

impl LinkPlatform {
    pub const fn is_donation(self) -> bool {
        matches!(
            self,
            Self::Patreon
                | Self::Bmac
                | Self::Paypal
                | Self::Github
                | Self::Kofi
                | Self::Other
        )
    }
}
