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
#[display(rename_all = "kebab-case")]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum LinkPlatform {
    Patreon,
    Bmac,
    Paypal,
    Github,
    #[display("ko-fi")]
    KoFi,
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
                | Self::KoFi
                | Self::Other
        )
    }
}
