use {
    crate::CurrencyCode,
    chrono::{DateTime, Utc},
    derive_more::{Deref, Display},
    serde::{Deserialize, Serialize},
    std::str::FromStr,
    uuid::Uuid,
};

#[cfg(feature = "client")]
const _: () = {
    use crate::{MuralError, RequestExt, SearchResponse};

    impl crate::Client {
        pub async fn search_organizations(
            &self,
            req: SearchRequest,
        ) -> Result<SearchResponse<OrganizationId, Organization>, MuralError> {
            #[derive(Debug, Serialize)]
            #[serde(rename_all = "camelCase")]
            struct Body {
                #[serde(skip_serializing_if = "Option::is_none")]
                filter: Option<Filter>,
            }

            #[derive(Debug, Serialize)]
            #[serde(rename_all = "camelCase")]
            struct Filter {
                #[serde(rename = "type")]
                ty: FilterType,
                name: String,
            }

            #[derive(Debug, Clone, Copy, Serialize)]
            #[serde(rename_all = "snake_case")]
            pub enum FilterType {
                Name,
            }

            maybe_mock!(self, search_organizations(req.clone()));

            let query = [
                req.limit.map(|limit| ("limit", limit.to_string())),
                req.next_id
                    .map(|next_id| ("nextId", next_id.hyphenated().to_string())),
            ]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();

            let body = Body {
                filter: req.name.map(|name| Filter {
                    ty: FilterType::Name,
                    name,
                }),
            };

            self.http_post(|base| format!("{base}/api/organizations/search"))
                .query(&query)
                .json(&body)
                .send_mural()
                .await
        }

        pub async fn get_organization(
            &self,
            id: OrganizationId,
        ) -> Result<Organization, MuralError> {
            maybe_mock!(self, get_organization(id));

            self.http_post(|base| format!("{base}/api/organizations/{id}"))
                .send_mural()
                .await
        }
    }
};

#[derive(Debug, Display, Clone, Copy, PartialEq, Eq, Hash, Deref, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[display("{}", _0.hyphenated())]
pub struct OrganizationId(pub Uuid);

impl FromStr for OrganizationId {
    type Err = <Uuid as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse::<Uuid>().map(Self)
    }
}

impl From<OrganizationId> for Uuid {
    fn from(value: OrganizationId) -> Self {
        value.0
    }
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct SearchRequest {
    pub limit: Option<u64>,
    pub next_id: Option<Uuid>,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Organization {
    Individual(Individual),
    Business(Business),
    EndUserCustodialIndividual(EndUserCustodialIndividual),
    EndUserCustodialBusiness(EndUserCustodialBusiness),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct Individual {
    pub id: OrganizationId,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub first_name: String,
    pub last_name: String,
    pub tos_status: TosStatus,
    pub kyc_status: KycStatus,
    pub currency_capabilities: Vec<CurrencyCapability>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct Business {
    pub id: OrganizationId,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub name: String,
    pub tos_status: TosStatus,
    pub kyc_status: KycStatus,
    pub currency_capabilities: Vec<CurrencyCapability>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct EndUserCustodialIndividual {
    pub id: OrganizationId,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub first_name: String,
    pub last_name: String,
    pub approver: Approver,
    pub tos_status: TosStatus,
    pub kyc_status: KycStatus,
    pub currency_capabilities: Vec<CurrencyCapability>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct EndUserCustodialBusiness {
    pub id: OrganizationId,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub name: String,
    pub approver: Approver,
    pub tos_status: TosStatus,
    pub kyc_status: KycStatus,
    pub currency_capabilities: Vec<CurrencyCapability>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct Approver {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub name: String,
    pub email: String,
    pub auth_methods: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(tag = "type", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TosStatus {
    NotAccepted,
    NeedsReview,
    Accepted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum KycStatus {
    Inactive,
    Pending,
    Approved {
        approved_at: DateTime<Utc>,
    },
    Errored {
        details: String,
        errored_at: DateTime<Utc>,
    },
    Rejected {
        reason: String,
        rejected_at: DateTime<Utc>,
    },
    PreValidationFailed {
        failed_validation_reason: FailedValidationReason,
        failed_validation_at: DateTime<Utc>,
    },
    NeedsUpdate {
        needs_update_reason: String,
        verification_status_updated_at: DateTime<Utc>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum FailedValidationReason {
    DocumentPrevalidationFailed {
        document_id: String,
        failed_validation_reason: String,
    },
    UltimateBeneficialOwnerPrevalidationFailed {
        ultimate_beneficial_owner_id: String,
        failed_validation_reason: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct CurrencyCapability {
    pub fiat_and_rail_code: String,
    pub currency_code: CurrencyCode,
    pub deposit_status: TransactionCapabilityStatus,
    pub pay_out_status: TransactionCapabilityStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum TransactionCapabilityStatus {
    TermsOfService {
        details: String,
    },
    #[serde(rename = "awaitingKYC")]
    AwaitingKyc {
        details: String,
    },
    Enabled,
    Rejected {
        reason: RejectedReason,
        details: String,
    },
    Disabled {
        reason: DisabledReason,
        details: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(tag = "type", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RejectedReason {
    KycFailed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(tag = "type", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DisabledReason {
    CapabilityUnavailable,
    ProcessingError,
}
