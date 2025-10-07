use derive_more::{Deref, Display};
use secrecy::ExposeSecret;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{CurrencyCode, DateTime, MuralPay, SearchResponse};

impl MuralPay {
    pub async fn search_organizations(
        &self,
        req: SearchRequest,
    ) -> reqwest::Result<SearchResponse<OrganizationId, Organization>> {
        #[derive(Debug, Serialize)]
        struct Body {
            #[serde(skip_serializing_if = "Option::is_none")]
            filter: Option<Filter>,
        }

        #[derive(Debug, Serialize)]
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

        self.http
            .post(format!("{}/api/organizations/search", self.api_url))
            .bearer_auth(self.api_key.expose_secret())
            .query(&query)
            .json(&body)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await
    }

    pub async fn get_organization(
        &self,
        id: OrganizationId,
    ) -> reqwest::Result<Organization> {
        self.http
            .post(format!("{}/api/organizations/{id}", self.api_url))
            .bearer_auth(self.api_key.expose_secret())
            .send()
            .await?
            .error_for_status()?
            .json()
            .await
    }
}

#[derive(
    Debug,
    Display,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    Deref,
    Serialize,
    Deserialize,
)]
#[display("{}", _0.hyphenated())]
pub struct OrganizationId(pub Uuid);

#[derive(Debug, Clone, Default)]
pub struct SearchRequest {
    pub limit: Option<u64>,
    pub next_id: Option<Uuid>,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Organization {
    Individual(Individual),
    Business(Business),
    EndUserCustodialIndividual(EndUserCustodialIndividual),
    EndUserCustodialBusiness(EndUserCustodialBusiness),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Individual {
    pub id: OrganizationId,
    pub created_at: DateTime,
    pub updated_at: DateTime,
    pub first_name: String,
    pub last_name: String,
    pub tos_status: TosStatus,
    pub kyc_status: KycStatus,
    pub currency_capabilities: Vec<CurrencyCapability>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Business {
    pub id: OrganizationId,
    pub created_at: DateTime,
    pub updated_at: DateTime,
    pub name: String,
    pub tos_status: TosStatus,
    pub kyc_status: KycStatus,
    pub currency_capabilities: Vec<CurrencyCapability>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EndUserCustodialIndividual {
    pub id: OrganizationId,
    pub created_at: DateTime,
    pub updated_at: DateTime,
    pub first_name: String,
    pub last_name: String,
    pub approver: Approver,
    pub tos_status: TosStatus,
    pub kyc_status: KycStatus,
    pub currency_capabilities: Vec<CurrencyCapability>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EndUserCustodialBusiness {
    pub id: OrganizationId,
    pub created_at: DateTime,
    pub updated_at: DateTime,
    pub name: String,
    pub approver: Approver,
    pub tos_status: TosStatus,
    pub kyc_status: KycStatus,
    pub currency_capabilities: Vec<CurrencyCapability>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Approver {
    pub id: Uuid,
    pub created_at: DateTime,
    pub name: String,
    pub email: String,
    pub auth_methods: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TosStatus {
    NotAccepted,
    NeedsReview,
    Accepted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum KycStatus {
    Inactive,
    Pending,
    Approved {
        approved_at: DateTime,
    },
    Errored {
        details: String,
        errored_at: DateTime,
    },
    Rejected {
        reason: String,
        rejected_at: DateTime,
    },
    PreValidationFailed {
        failed_validation_reason: FailedValidationReason,
        failed_validation_at: DateTime,
    },
    NeedsUpdate {
        needs_update_reason: String,
        verification_status_updated_at: DateTime,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[serde(rename_all = "camelCase")]
pub struct CurrencyCapability {
    pub fiat_and_rail_code: String,
    pub currency_code: CurrencyCode,
    pub deposit_status: TransactionCapabilityStatus,
    pub pay_out_status: TransactionCapabilityStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[serde(tag = "type", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RejectedReason {
    KycFailed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DisabledReason {
    CapabilityUnavailable,
    ProcessingError,
}
