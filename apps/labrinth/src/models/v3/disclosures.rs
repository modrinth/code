use crate::database::models::DBProjectDisclosure;
use ariadne::ids::UserId;
use chrono::{DateTime, Utc};
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

impl ProjectDisclosure {
    pub fn to_parts(
        &self,
    ) -> Result<(&'static str, serde_json::Value), serde_json::Error> {
        let serde_json::Value::Object(mut object) = serde_json::to_value(self)?
        else {
            return Err(serde::ser::Error::custom(
                "project disclosure must serialize to a JSON object",
            ));
        };
        object.remove("type");
        Ok((self.into(), serde_json::Value::Object(object)))
    }

    pub fn from_parts(
        kind: &str,
        metadata: serde_json::Value,
    ) -> Result<Self, serde_json::Error> {
        let serde_json::Value::Object(mut object) = metadata else {
            return Err(serde::ser::Error::custom(
                "project disclosure metadata must be a JSON object, this should never be reachable",
            ));
        };
        object.insert(
            "type".to_owned(),
            serde_json::Value::String(kind.to_owned()),
        );
        serde_json::from_value(serde_json::Value::Object(object))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ProjectDisclosureData {
    #[serde(flatten)]
    pub disclosure: ProjectDisclosure,
    pub set_by_moderator: bool,
    pub updated_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_by: Option<UserId>,
}

impl ProjectDisclosureData {
    pub fn from_db(
        value: DBProjectDisclosure,
        viewer_is_moderator: bool,
    ) -> Self {
        let updated_by = (!value.set_by_moderator || viewer_is_moderator)
            .then_some(value.updated_by.into());

        Self {
            disclosure: value.disclosure,
            set_by_moderator: value.set_by_moderator,
            updated_at: value.updated_at,
            updated_by,
        }
    }
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
