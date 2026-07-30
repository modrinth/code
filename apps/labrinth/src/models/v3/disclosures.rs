use serde::{Deserialize, Serialize};
use strum::IntoStaticStr;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, IntoStaticStr)]
#[serde(rename_all = "snake_case", tag = "type")]
#[strum(serialize_all = "snake_case")]
pub enum ProjectDisclosure {
    AiContent {
        note: Option<String>,
    },
    Advertisements {
        note: Option<String>,
    },
    EpilepsyTriggers {
        note: Option<String>,
    },
    SystemInteractions {
        note: Option<String>,
    },
    Telemetry {
        consent: TelementryConsent,
        data_collected: Vec<String>,
    },
    DerivativeWork {
        sources: Vec<DerivativeSource>,
    },
    PaidFeatures {
        features: Vec<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum TelementryConsent {
    OptIn,
    OptOut,
    AlwaysActive,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DerivativeSource {
    pub link: Option<String>,
    pub label: String,
    pub note: Option<String>,
}
