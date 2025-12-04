#![cfg_attr(
    feature = "utoipa",
    expect(
        clippy::large_stack_arrays,
        reason = "due to `utoipa::ToSchema` derive"
    )
)]

use {
    crate::{
        AccountId, Blockchain, CounterpartyId, CurrencyCode, FiatAccountType,
        FiatAmount, FiatAndRailCode, PayoutMethodId, TokenAmount,
        TransactionId, WalletDetails,
    },
    chrono::{DateTime, Utc},
    derive_more::{Deref, Display, Error, From},
    rust_decimal::Decimal,
    rust_iso3166::CountryCode,
    serde::{Deserialize, Serialize},
    serde_with::{DeserializeFromStr, SerializeDisplay},
    std::str::FromStr,
    uuid::Uuid,
};

#[cfg(feature = "client")]
const _: () = {
    use crate::{MuralError, RequestExt, SearchParams, SearchResponse};

    impl crate::Client {
        pub async fn search_payout_requests(
            &self,
            filter: Option<PayoutStatusFilter>,
            params: Option<SearchParams<PayoutRequestId>>,
        ) -> Result<SearchResponse<PayoutRequestId, PayoutRequest>, MuralError>
        {
            #[derive(Debug, Serialize)]
            #[serde(rename_all = "camelCase")]
            struct Body {
                // if we submit `null`, Mural errors; we have to explicitly exclude this field
                #[serde(skip_serializing_if = "Option::is_none")]
                filter: Option<PayoutStatusFilter>,
            }

            maybe_mock!(self, search_payout_requests(filter, params));

            let body = Body { filter };

            self.http_post(|base| format!("{base}/api/payouts/search"))
                .query(&params.map(|p| p.to_query()).unwrap_or_default())
                .json(&body)
                .send_mural()
                .await
        }

        pub async fn get_payout_request(
            &self,
            id: PayoutRequestId,
        ) -> Result<PayoutRequest, MuralError> {
            maybe_mock!(self, get_payout_request(id));

            self.http_get(|base| format!("{base}/api/payouts/payout/{id}"))
                .send_mural()
                .await
        }

        pub async fn get_fees_for_token_amount(
            &self,
            token_fee_requests: &[TokenFeeRequest],
        ) -> Result<Vec<TokenPayoutFee>, MuralError> {
            #[derive(Debug, Serialize)]
            #[serde(rename_all = "camelCase")]
            struct Body<'a> {
                token_fee_requests: &'a [TokenFeeRequest],
            }

            maybe_mock!(self, get_fees_for_token_amount(token_fee_requests));

            let body = Body { token_fee_requests };

            self.http_post(|base| {
                format!("{base}/api/payouts/fees/token-to-fiat")
            })
            .json(&body)
            .send_mural()
            .await
        }

        pub async fn get_fees_for_fiat_amount(
            &self,
            fiat_fee_requests: &[FiatFeeRequest],
        ) -> Result<Vec<FiatPayoutFee>, MuralError> {
            #[derive(Debug, Serialize)]
            #[serde(rename_all = "camelCase")]
            struct Body<'a> {
                fiat_fee_requests: &'a [FiatFeeRequest],
            }

            maybe_mock!(self, get_fees_for_fiat_amount(fiat_fee_requests));

            let body = Body { fiat_fee_requests };

            self.http_post(|base| {
                format!("{base}/api/payouts/fees/fiat-to-token")
            })
            .json(&body)
            .send_mural()
            .await
        }

        pub async fn create_payout_request(
            &self,
            source_account_id: AccountId,
            memo: Option<impl AsRef<str>>,
            payouts: &[CreatePayout],
        ) -> Result<PayoutRequest, MuralError> {
            #[derive(Debug, Serialize)]
            #[serde(rename_all = "camelCase")]
            struct Body<'a> {
                source_account_id: AccountId,
                memo: Option<&'a str>,
                payouts: &'a [CreatePayout],
            }

            maybe_mock!(
                self,
                create_payout_request(
                    source_account_id,
                    memo.as_ref().map(AsRef::as_ref),
                    payouts
                )
            );

            let body = Body {
                source_account_id,
                memo: memo.as_ref().map(AsRef::as_ref),
                payouts,
            };

            self.http_post(|base| format!("{base}/api/payouts/payout"))
                .json(&body)
                .send_mural()
                .await
        }

        pub async fn execute_payout_request(
            &self,
            id: PayoutRequestId,
        ) -> Result<PayoutRequest, MuralError> {
            maybe_mock!(self, execute_payout_request(id));

            self.http_post(|base| {
                format!("{base}/api/payouts/payout/{id}/execute")
            })
            .transfer_auth(self)
            .send_mural()
            .await
        }

        pub async fn cancel_payout_request(
            &self,
            id: PayoutRequestId,
        ) -> Result<PayoutRequest, MuralError> {
            maybe_mock!(self, cancel_payout_request(id));

            self.http_post(|base| {
                format!("{base}/api/payouts/payout/{id}/cancel")
            })
            .transfer_auth(self)
            .send_mural()
            .await
        }

        pub async fn get_bank_details(
            &self,
            fiat_currency_and_rail: &[FiatAndRailCode],
        ) -> Result<BankDetailsResponse, MuralError> {
            maybe_mock!(self, get_bank_details(fiat_currency_and_rail));

            let query = fiat_currency_and_rail
                .iter()
                .map(|code| ("fiatCurrencyAndRail", code.to_string()))
                .collect::<Vec<_>>();

            self.http_get(|base| format!("{base}/api/payouts/bank-details"))
                .query(&query)
                .send_mural()
                .await
        }
    }
};

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
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[display("{}", _0.hyphenated())]
pub struct PayoutRequestId(pub Uuid);

impl FromStr for PayoutRequestId {
    type Err = <Uuid as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse::<Uuid>().map(Self)
    }
}

impl From<PayoutRequestId> for Uuid {
    fn from(value: PayoutRequestId) -> Self {
        value.0
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
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[display("{}", _0.hyphenated())]
pub struct PayoutId(pub Uuid);

impl FromStr for PayoutId {
    type Err = <Uuid as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse::<Uuid>().map(Self)
    }
}

impl From<PayoutId> for Uuid {
    fn from(value: PayoutId) -> Self {
        value.0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum PayoutStatusFilter {
    PayoutStatus { statuses: Vec<PayoutStatus> },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct PayoutRequest {
    pub id: PayoutRequestId,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub source_account_id: AccountId,
    pub transaction_hash: Option<String>,
    pub memo: Option<String>,
    pub status: PayoutStatus,
    pub payouts: Vec<Payout>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PayoutStatus {
    AwaitingExecution,
    Canceled,
    Pending,
    Executed,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct Payout {
    pub id: PayoutId,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub amount: TokenAmount,
    pub details: PayoutDetails,
    pub recipient_info: PayoutRecipientInfo,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum PayoutDetails {
    Fiat(FiatPayoutDetails),
    Blockchain(BlockchainPayoutDetails),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct FiatPayoutDetails {
    pub fiat_and_rail_code: FiatAndRailCode,
    pub fiat_payout_status: FiatPayoutStatus,
    pub fiat_amount: FiatAmount,
    pub transaction_fee: TokenAmount,
    #[serde(with = "rust_decimal::serde::float")]
    pub exchange_fee_percentage: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub exchange_rate: Decimal,
    pub fee_total: TokenAmount,
    pub developer_fee: Option<DeveloperFee>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum FiatPayoutStatus {
    Created,
    #[serde(rename_all = "camelCase")]
    Pending {
        initiated_at: DateTime<Utc>,
    },
    #[serde(rename_all = "camelCase")]
    OnHold {
        initiated_at: DateTime<Utc>,
    },
    #[serde(rename_all = "camelCase")]
    Completed {
        initiated_at: DateTime<Utc>,
        completed_at: DateTime<Utc>,
    },
    #[serde(rename_all = "camelCase")]
    Failed {
        initiated_at: DateTime<Utc>,
        reason: String,
        error_code: FiatPayoutErrorCode,
    },
    #[serde(rename_all = "camelCase")]
    Canceled,
    // since 1.31
    #[serde(rename_all = "camelCase")]
    RefundInProgress {
        error_code: RefundErrorCode,
        failure_reason: String,
        refund_initiated_at: DateTime<Utc>,
    },
    // since 1.31
    #[serde(rename_all = "camelCase")]
    Refunded {
        error_code: RefundErrorCode,
        failure_reason: String,
        refund_completed_at: DateTime<Utc>,
        refund_initiated_at: DateTime<Utc>,
        refund_transaction_id: TransactionId,
    },
}

// since 1.31
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(tag = "type", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RefundErrorCode {
    Unknown,
    AccountNumberIncorrect,
    RejectedByBank,
    AccountTypeIncorrect,
    AccountClosed,
    BeneficiaryDocumentationIncorrect,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum FiatPayoutStatusKind {
    Created,
    Pending,
    OnHold,
    Completed,
    Failed,
    Canceled,
    RefundInProgress,
    Refunded,
}

impl FiatPayoutStatus {
    #[must_use]
    pub const fn kind(&self) -> FiatPayoutStatusKind {
        match self {
            Self::Created { .. } => FiatPayoutStatusKind::Created,
            Self::Pending { .. } => FiatPayoutStatusKind::Pending,
            Self::OnHold { .. } => FiatPayoutStatusKind::OnHold,
            Self::Completed { .. } => FiatPayoutStatusKind::Completed,
            Self::Failed { .. } => FiatPayoutStatusKind::Failed,
            Self::Canceled { .. } => FiatPayoutStatusKind::Canceled,
            Self::RefundInProgress { .. } => {
                FiatPayoutStatusKind::RefundInProgress
            }
            Self::Refunded { .. } => FiatPayoutStatusKind::Refunded,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FiatPayoutErrorCode {
    Unknown,
    AccountNumberIncorrect,
    RejectedByBank,
    AccountTypeIncorrect,
    AccountClosed,
    BeneficiaryDocumentationIncorrect,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct DeveloperFee {
    #[serde(with = "rust_decimal::serde::float_option", default)]
    pub developer_fee_percentage: Option<Decimal>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct BlockchainPayoutDetails {
    pub wallet_address: String,
    pub blockchain: Blockchain,
    pub status: BlockchainPayoutStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BlockchainPayoutStatus {
    AwaitingExecution,
    Pending,
    Executed,
    Failed,
    Canceled,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum PayoutRecipientInfo {
    #[serde(rename_all = "camelCase")]
    Counterparty {
        counterparty_id: CounterpartyId,
        payout_method_id: PayoutMethodId,
    },
    #[serde(rename_all = "camelCase")]
    Inline {
        name: String,
        details: InlineRecipientDetails,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum InlineRecipientDetails {
    #[serde(rename_all = "camelCase")]
    Fiat { details: InlineFiatRecipientDetails },
    #[serde(rename_all = "camelCase")]
    Blockchain {
        wallet_address: String,
        blockchain: Blockchain,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct InlineFiatRecipientDetails {
    pub fiat_currency_code: CurrencyCode,
    pub bank_name: String,
    pub truncated_bank_account_number: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct CreatePayout {
    pub amount: TokenAmount,
    pub payout_details: CreatePayoutDetails,
    pub recipient_info: CreatePayoutRecipientInfo,
    pub supporting_details: Option<SupportingDetails>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum CreatePayoutDetails {
    #[serde(rename_all = "camelCase")]
    Fiat {
        bank_name: String,
        bank_account_owner: String,
        developer_fee: Option<DeveloperFee>,
        fiat_and_rail_details: FiatAndRailDetails,
    },
    #[serde(rename_all = "camelCase")]
    Blockchain { wallet_details: WalletDetails },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum FiatAndRailDetails {
    #[serde(rename_all = "camelCase")]
    Usd {
        symbol: UsdSymbol,
        account_type: FiatAccountType,
        bank_account_number: String,
        bank_routing_number: String,
    },
    #[serde(rename_all = "camelCase")]
    Cop {
        symbol: CopSymbol,
        phone_number: String,
        account_type: FiatAccountType,
        bank_account_number: String,
        document_number: String,
        document_type: DocumentType,
    },
    #[serde(rename_all = "camelCase")]
    Ars {
        symbol: ArsSymbol,
        bank_account_number: String,
        document_number: String,
        bank_account_number_type: String,
    },
    #[serde(rename_all = "camelCase")]
    Eur {
        symbol: EurSymbol,
        iban: String,
        swift_bic: String,
        #[serde(with = "crate::serde_iso3166")]
        #[cfg_attr(feature = "utoipa", schema(value_type = String))]
        country: CountryCode,
    },
    #[serde(rename_all = "camelCase")]
    Mxn {
        symbol: MxnSymbol,
        bank_account_number: String,
    },
    #[serde(rename_all = "camelCase")]
    Brl {
        symbol: BrlSymbol,
        pix_account_type: PixAccountType,
        pix_email: String,
        pix_phone: String,
        branch_code: String,
        document_number: String,
    },
    #[serde(rename_all = "camelCase")]
    Clp {
        symbol: ClpSymbol,
        account_type: FiatAccountType,
        bank_account_number: String,
        document_type: DocumentType,
        document_number: String,
    },
    #[serde(rename_all = "camelCase")]
    Pen {
        symbol: PenSymbol,
        document_number: String,
        document_type: DocumentType,
        bank_account_number: String,
        account_type: FiatAccountType,
    },
    #[serde(rename_all = "camelCase")]
    Bob {
        symbol: BobSymbol,
        bank_account_number: String,
        document_number: String,
        document_type: DocumentType,
    },
    #[serde(rename_all = "camelCase")]
    Crc {
        symbol: CrcSymbol,
        iban: String,
        document_number: String,
        document_type: DocumentType,
    },
    #[serde(rename_all = "camelCase")]
    Zar {
        symbol: ZarSymbol,
        account_type: FiatAccountType,
        bank_account_number: String,
    },
    #[serde(rename_all = "camelCase")]
    UsdPeru {
        symbol: UsdSymbol,
        account_type: FiatAccountType,
        bank_account_number: String,
        document_number: String,
        document_type: DocumentType,
    },
    #[serde(rename_all = "camelCase")]
    UsdChina {
        symbol: UsdSymbol,
        bank_name: String,
        account_type: FiatAccountType,
        bank_account_number: String,
        document_number: String,
        document_type: DocumentType,
        phone_number: String,
        address: String,
        swift_bic: String,
    },
}

impl FiatAndRailDetails {
    #[must_use]
    pub const fn code(&self) -> FiatAndRailCode {
        match self {
            Self::Usd { .. } => FiatAndRailCode::Usd,
            Self::Cop { .. } => FiatAndRailCode::Cop,
            Self::Ars { .. } => FiatAndRailCode::Ars,
            Self::Eur { .. } => FiatAndRailCode::Eur,
            Self::Mxn { .. } => FiatAndRailCode::Mxn,
            Self::Brl { .. } => FiatAndRailCode::Brl,
            Self::Clp { .. } => FiatAndRailCode::Clp,
            Self::Pen { .. } => FiatAndRailCode::Pen,
            Self::Bob { .. } => FiatAndRailCode::Bob,
            Self::Crc { .. } => FiatAndRailCode::Crc,
            Self::Zar { .. } => FiatAndRailCode::Zar,
            Self::UsdPeru { .. } => FiatAndRailCode::UsdPeru,
            Self::UsdChina { .. } => FiatAndRailCode::UsdChina,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(rename_all = "SCREAMING-KEBAB-CASE")]
pub enum UsdSymbol {
    Usd,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(rename_all = "SCREAMING-KEBAB-CASE")]
pub enum CopSymbol {
    Cop,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(rename_all = "SCREAMING-KEBAB-CASE")]
pub enum ArsSymbol {
    Ars,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(rename_all = "SCREAMING-KEBAB-CASE")]
pub enum EurSymbol {
    Eur,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(rename_all = "SCREAMING-KEBAB-CASE")]
pub enum MxnSymbol {
    Mxn,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(rename_all = "SCREAMING-KEBAB-CASE")]
pub enum BrlSymbol {
    Brl,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(rename_all = "SCREAMING-KEBAB-CASE")]
pub enum ClpSymbol {
    Clp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(rename_all = "SCREAMING-KEBAB-CASE")]
pub enum PenSymbol {
    Pen,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(rename_all = "SCREAMING-KEBAB-CASE")]
pub enum BobSymbol {
    Bob,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(rename_all = "SCREAMING-KEBAB-CASE")]
pub enum CrcSymbol {
    Crc,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(rename_all = "SCREAMING-KEBAB-CASE")]
pub enum ZarSymbol {
    Zar,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DocumentType {
    NationalId,
    Passport,
    ResidentId,
    Ruc,
    TaxId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PixAccountType {
    Phone,
    Email,
    Document,
    BankAccount,
}

#[derive(Debug, Clone, Serialize, Deserialize, From)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum CreatePayoutRecipientInfo {
    #[serde(rename_all = "camelCase")]
    Individual {
        first_name: String,
        last_name: String,
        email: String,
        date_of_birth: Dob,
        physical_address: PhysicalAddress,
    },
    #[serde(rename_all = "camelCase")]
    Business {
        name: String,
        email: String,
        physical_address: PhysicalAddress,
    },
}

impl CreatePayoutRecipientInfo {
    #[must_use]
    pub fn email(&self) -> &str {
        match self {
            Self::Individual { email, .. } | Self::Business { email, .. } => {
                email
            }
        }
    }

    #[must_use]
    pub const fn physical_address(&self) -> &PhysicalAddress {
        match self {
            Self::Individual {
                physical_address, ..
            }
            | Self::Business {
                physical_address, ..
            } => physical_address,
        }
    }
}

#[derive(Debug, Display, Clone, Copy, SerializeDisplay, DeserializeFromStr)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[display("{year:04}-{month:02}-{day:02}")]
pub struct Dob {
    year: u16,
    month: u8,
    day: u8,
}

#[derive(Debug, Display, Clone, Error)]
pub enum InvalidDob {
    #[display("must be three segments separated by `-`")]
    NotThreeSegments,
    #[display("year is not an integer")]
    YearNotInt,
    #[display("month is not an integer")]
    MonthNotInt,
    #[display("day is not an integer")]
    DayNotInt,
    #[display("year out of range")]
    YearRange,
    #[display("month out of range")]
    MonthRange,
    #[display("day out of range")]
    DayRange,
}

impl Dob {
    pub fn new(year: u16, month: u8, day: u8) -> Result<Self, InvalidDob> {
        if !(1000..10000).contains(&year) {
            return Err(InvalidDob::YearRange);
        }
        if month > 12 {
            return Err(InvalidDob::MonthRange);
        }
        if day > 31 {
            return Err(InvalidDob::DayRange);
        }
        Ok(Self { year, month, day })
    }
}

impl FromStr for Dob {
    type Err = InvalidDob;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let [year, month, day] = s
            .split('-')
            .collect::<Vec<_>>()
            .try_into()
            .map_err(|_| InvalidDob::NotThreeSegments)?;
        let year = year.parse::<u16>().map_err(|_| InvalidDob::YearNotInt)?;
        let month = month.parse::<u8>().map_err(|_| InvalidDob::MonthNotInt)?;
        let day = day.parse::<u8>().map_err(|_| InvalidDob::DayNotInt)?;
        Self::new(year, month, day)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct PhysicalAddress {
    pub address1: String,
    pub address2: Option<String>,
    #[serde(with = "crate::serde_iso3166")]
    #[cfg_attr(feature = "utoipa", schema(value_type = String))]
    pub country: CountryCode,
    pub state: String,
    pub city: String,
    pub zip: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct SupportingDetails {
    pub supporting_document: Option<String>, // data:image/jpeg;base64,...
    pub payout_purpose: Option<PayoutPurpose>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PayoutPurpose {
    VendorPayment,
    Payroll,
    TaxPayment,
    RentLeasePayment,
    SupplierPayment,
    PersonalGift,
    FamilySupport,
    CharitableDonation,
    ExpenseReimbursement,
    BillUtilityPayment,
    TravelExpenses,
    InvestmentContribution,
    CashWithdrawal,
    RealEstatePurchase,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct TokenFeeRequest {
    pub amount: TokenAmount,
    pub fiat_and_rail_code: FiatAndRailCode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum TokenPayoutFee {
    #[serde(rename_all = "camelCase")]
    Success {
        #[serde(with = "rust_decimal::serde::float")]
        exchange_rate: Decimal,
        #[serde(with = "rust_decimal::serde::float")]
        exchange_fee_percentage: Decimal,
        fiat_and_rail_code: FiatAndRailCode,
        transaction_fee: TokenAmount,
        min_transaction_value: TokenAmount,
        estimated_fiat_amount: FiatAmount,
        token_amount: TokenAmount,
        fee_total: TokenAmount,
    },
    #[serde(rename_all = "camelCase")]
    Error {
        token_amount: TokenAmount,
        message: String,
        fiat_and_rail_code: FiatAndRailCode,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct FiatFeeRequest {
    #[serde(with = "rust_decimal::serde::float")]
    pub fiat_amount: Decimal,
    pub token_symbol: String,
    pub fiat_and_rail_code: FiatAndRailCode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum FiatPayoutFee {
    #[serde(rename_all = "camelCase")]
    Success {
        token_symbol: String,
        fiat_amount: FiatAmount,
        #[serde(with = "rust_decimal::serde::float")]
        exchange_rate: Decimal,
        #[serde(with = "rust_decimal::serde::float")]
        exchange_fee_percentage: Decimal,
        fiat_and_rail_code: FiatAndRailCode,
        transaction_fee: TokenAmount,
        min_transaction_value: TokenAmount,
        estimated_token_amount_required: TokenAmount,
        fee_total: TokenAmount,
    },
    #[serde(rename_all = "camelCase")]
    Error {
        message: String,
        fiat_and_rail_code: FiatAndRailCode,
        token_symbol: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct BankDetailsResponse {
    pub bank_details: CurrenciesBankDetails,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(rename_all = "kebab-case")]
pub struct CurrenciesBankDetails {
    #[serde(default)]
    pub usd: CurrencyBankDetails,
    #[serde(default)]
    pub cop: CurrencyBankDetails,
    #[serde(default)]
    pub ars: CurrencyBankDetails,
    #[serde(default)]
    pub eur: CurrencyBankDetails,
    #[serde(default)]
    pub mxn: CurrencyBankDetails,
    #[serde(default)]
    pub brl: CurrencyBankDetails,
    #[serde(default)]
    pub clp: CurrencyBankDetails,
    #[serde(default)]
    pub pen: CurrencyBankDetails,
    #[serde(default)]
    pub bob: CurrencyBankDetails,
    #[serde(default)]
    pub crc: CurrencyBankDetails,
    #[serde(default)]
    pub zar: CurrencyBankDetails,
    #[serde(default)]
    pub usd_peru: CurrencyBankDetails,
    #[serde(default)]
    pub usd_china: CurrencyBankDetails,
    #[serde(default)]
    pub usd_panama: CurrencyBankDetails,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct CurrencyBankDetails {
    pub bank_names: Vec<String>,
}
