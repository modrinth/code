use std::str::FromStr;

use derive_more::{Deref, Display, Error};
use secrecy::ExposeSecret;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use uuid::Uuid;

use crate::{
    AccountId, Blockchain, DateTime, FiatAmount, FiatAndRailCode, MuralPay,
    SearchParams, SearchResponse, TokenAmount, WalletDetails,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum PayoutStatusFilter {
    PayoutStatus { statuses: Vec<String> },
}

impl MuralPay {
    pub async fn search_payout_requests(
        &self,
        filter: Option<PayoutStatusFilter>,
        params: Option<SearchParams<PayoutRequestId>>,
    ) -> reqwest::Result<SearchResponse<PayoutRequestId, PayoutRequest>> {
        #[derive(Debug, Serialize)]
        struct Body {
            filter: Option<PayoutStatusFilter>,
        }

        let body = Body { filter };

        self.http
            .post(format!("{}/api/payouts/search", self.api_url))
            .bearer_auth(self.api_key.expose_secret())
            .query(&params.map(|p| p.to_query()).unwrap_or_default())
            .json(&body)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await
    }

    pub async fn get_payout_request(
        &self,
        id: PayoutRequestId,
    ) -> reqwest::Result<PayoutRequest> {
        self.http
            .get(format!("{}/api/payouts/payout/{id}", self.api_url))
            .bearer_auth(self.api_key.expose_secret())
            .send()
            .await?
            .error_for_status()?
            .json()
            .await
    }

    pub async fn create_payout_request(
        &self,
        source_account_id: AccountId,
        memo: Option<impl AsRef<str>>,
        payouts: impl IntoIterator<Item = CreatePayout>,
    ) -> reqwest::Result<PayoutRequest> {
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
pub struct PayoutRequestId(pub Uuid);

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
pub struct PayoutId(pub Uuid);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PayoutRequest {
    pub id: PayoutRequestId,
    pub created_at: DateTime,
    pub updated_at: DateTime,
    pub source_account_id: AccountId,
    pub transaction_hash: Option<String>,
    pub memo: Option<String>,
    pub status: PayoutStatus,
    pub payouts: Vec<Payout>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PayoutStatus {
    AwaitingExecution,
    Canceled,
    Pending,
    Executed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Payout {
    pub id: PayoutId,
    pub created_at: DateTime,
    pub updated_at: DateTime,
    pub amount: PayoutAmount,
    pub details: PayoutDetails,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PayoutAmount {
    pub token_amount: u64,
    pub token_symbol: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum PayoutDetails {
    Fiat(FiatPayoutDetails),
    Blockchain(BlockchainPayoutDetails),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FiatPayoutDetails {
    pub fiat_and_rail_code: FiatAndRailCode,
    pub fiat_payout_status: FiatPayoutStatus,
    pub fiat_amount: FiatAmount,
    pub transaction_fee: TokenAmount,
    pub exchange_fee_percentage: f64,
    pub exchange_rate: f64,
    pub fee_total: TokenAmount,
    pub developer_fee: Option<DeveloperFee>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FiatPayoutStatus {
    Created,
    #[serde(rename_all = "camelCase")]
    Pending {
        initiated_at: DateTime,
    },
    #[serde(rename_all = "camelCase")]
    OnHold {
        initiated_at: DateTime,
    },
    #[serde(rename_all = "camelCase")]
    Completed {
        initiated_at: DateTime,
        completed_at: DateTime,
    },
    #[serde(rename_all = "camelCase")]
    Failed {
        initiated_at: DateTime,
        reason: String,
        error_code: FiatPayoutErrorCode,
    },
    Canceled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FiatPayoutErrorCode {
    Unknown,
    AccountNumberIncorrect,
    RejectedByBank,
    AccountTypeIncorrect,
    AccountClosed,
    BeneficiaryDocumentationIncorrect,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeveloperFee {
    pub developer_fee_percentage: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockchainPayoutDetails {
    pub wallet_address: String,
    pub blockchain: Blockchain,
    pub status: BlockchainPayoutStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BlockchainPayoutStatus {
    AwaitingExecution,
    Pending,
    Executed,
    Failed,
    Canceled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatePayout {
    pub amount: TokenAmount,
    pub payout_details: PayoutCreate,
    pub recipient_info: PayoutRecipientInfo,
    pub supporting_details: Option<SupportingDetails>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum PayoutCreate {
    Fiat {
        bank_name: String,
        bank_account_owner: String,
        developer_fee: Option<DeveloperFee>,
        fiat_and_rail_details: FiatAndRailDetails,
    },
    Blockchain {
        wallet_details: WalletDetails,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
        country: String,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING-KEBAB-CASE")]
pub enum UsdSymbol {
    Usd,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING-KEBAB-CASE")]
pub enum CopSymbol {
    Cop,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING-KEBAB-CASE")]
pub enum ArsSymbol {
    Ars,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING-KEBAB-CASE")]
pub enum EurSymbol {
    Eur,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING-KEBAB-CASE")]
pub enum MxnSymbol {
    Mxn,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING-KEBAB-CASE")]
pub enum BrlSymbol {
    Brl,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING-KEBAB-CASE")]
pub enum ClpSymbol {
    Clp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING-KEBAB-CASE")]
pub enum PenSymbol {
    Pen,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING-KEBAB-CASE")]
pub enum BobSymbol {
    Bob,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING-KEBAB-CASE")]
pub enum CrcSymbol {
    Crc,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING-KEBAB-CASE")]
pub enum ZarSymbol {
    Zar,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING-KEBAB-CASE")]
pub enum FiatAccountType {
    Checking,
    Savings,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DocumentType {
    NationalId,
    Passport,
    ResidentId,
    Ruc,
    TaxId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PixAccountType {
    Phone,
    Email,
    Document,
    BankAccount,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PayoutRecipientInfo {
    Individual {
        first_name: String,
        last_name: String,
        email: String,
        date_of_birth: Dob,
    },
}

#[derive(Debug, Display, Clone, Copy)]
#[display("{year}-{month}-{day}")]
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
    #[display("month out of range")]
    MonthRange,
    #[display("day out of range")]
    DayRange,
}

impl Dob {
    pub fn new(year: u16, month: u8, day: u8) -> Result<Self, InvalidDob> {
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
