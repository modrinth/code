use {
    crate::PhysicalAddress,
    chrono::{DateTime, Utc},
    derive_more::{Deref, Display},
    serde::{Deserialize, Serialize},
    std::str::FromStr,
    uuid::Uuid,
};

#[cfg(feature = "client")]
const _: () = {
    use crate::{MuralError, RequestExt, SearchParams, SearchResponse};

    impl crate::Client {
        pub async fn search_counterparties(
            &self,
            params: Option<SearchParams<CounterpartyId>>,
        ) -> Result<SearchResponse<CounterpartyId, Counterparty>, MuralError> {
            maybe_mock!(self, search_counterparties(params));

            self.http_post(|base| format!("{base}/api/counterparties/search"))
                .query(&params.map(|p| p.to_query()).unwrap_or_default())
                .send_mural()
                .await
        }

        pub async fn get_counterparty(
            &self,
            id: CounterpartyId,
        ) -> Result<Counterparty, MuralError> {
            maybe_mock!(self, get_counterparty(id));

            self.http_get(|base| format!("{base}/api/counterparties/counterparty/{id}"))
                .send_mural()
                .await
        }

        pub async fn create_counterparty(
            &self,
            counterparty: &CreateCounterparty,
        ) -> Result<Counterparty, MuralError> {
            #[derive(Debug, Serialize)]
            #[serde(rename_all = "camelCase")]
            struct Body<'a> {
                counterparty: &'a CreateCounterparty,
            }

            maybe_mock!(self, create_counterparty(counterparty));

            let body = Body { counterparty };

            self.http_post(|base| format!("{base}/api/counterparties"))
                .json(&body)
                .send_mural()
                .await
        }

        pub async fn update_counterparty(
            &self,
            id: CounterpartyId,
            counterparty: &UpdateCounterparty,
        ) -> Result<Counterparty, MuralError> {
            #[derive(Debug, Serialize)]
            #[serde(rename_all = "camelCase")]
            struct Body<'a> {
                counterparty: &'a UpdateCounterparty,
            }

            maybe_mock!(self, update_counterparty(id, counterparty));

            let body = Body { counterparty };

            self.http_put(|base| format!("{base}/api/counterparties/counterparty/{id}"))
                .json(&body)
                .send_mural()
                .await
        }
    }
};

#[derive(Debug, Display, Clone, Copy, PartialEq, Eq, Hash, Deref, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[display("{}", _0.hyphenated())]
pub struct CounterpartyId(pub Uuid);

impl FromStr for CounterpartyId {
    type Err = <Uuid as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse::<Uuid>().map(Self)
    }
}

impl From<CounterpartyId> for Uuid {
    fn from(value: CounterpartyId) -> Self {
        value.0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct Counterparty {
    pub id: CounterpartyId,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub alias: Option<String>,
    #[serde(flatten)]
    pub kind: CounterpartyKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum CounterpartyKind {
    #[serde(rename_all = "camelCase")]
    Individual {
        first_name: String,
        last_name: String,
        email: String,
        physical_address: PhysicalAddress,
    },
    #[serde(rename_all = "camelCase")]
    Business {
        name: String,
        email: String,
        physical_address: PhysicalAddress,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum CreateCounterparty {
    #[serde(rename_all = "camelCase")]
    Individual {
        alias: Option<String>,
        first_name: String,
        last_name: String,
        email: String,
        physical_address: PhysicalAddress,
    },
    #[serde(rename_all = "camelCase")]
    Business {
        alias: Option<String>,
        name: String,
        email: String,
        physical_address: PhysicalAddress,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum UpdateCounterparty {
    #[serde(rename_all = "camelCase")]
    Individual {
        alias: Option<String>,
        first_name: Option<String>,
        last_name: Option<String>,
        email: Option<String>,
        physical_address: Option<PhysicalAddress>,
    },
    #[serde(rename_all = "camelCase")]
    Business {
        alias: Option<String>,
        name: Option<String>,
        email: Option<String>,
        physical_address: Option<PhysicalAddress>,
    },
}
