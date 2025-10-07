use derive_more::{Deref, Display};
use secrecy::ExposeSecret;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    Blockchain, DateTime, FiatAmount, MuralPay, TokenAmount, WalletDetails,
};

impl MuralPay {
    pub async fn get_all_accounts(&self) -> reqwest::Result<Vec<Account>> {
        self.http
            .get(format!("{}/api/accounts", self.api_url))
            .bearer_auth(self.api_key.expose_secret())
            .send()
            .await?
            .error_for_status()?
            .json()
            .await
    }

    pub async fn get_account(&self, id: AccountId) -> reqwest::Result<Account> {
        self.http
            .get(format!("{}/api/accounts/{id}", self.api_url))
            .bearer_auth(self.api_key.expose_secret())
            .send()
            .await?
            .error_for_status()?
            .json()
            .await
    }

    pub async fn create_account(
        &self,
        name: impl AsRef<str>,
        description: Option<impl AsRef<str>>,
    ) -> reqwest::Result<Account> {
        #[derive(Debug, Serialize)]
        struct Body<'a> {
            name: &'a str,
            description: Option<&'a str>,
        }

        let body = Body {
            name: name.as_ref(),
            description: description.as_ref().map(|x| x.as_ref()),
        };

        self.http
            .post(format!("{}/api/accounts", self.api_url))
            .bearer_auth(self.api_key.expose_secret())
            .json(&body)
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
pub struct AccountId(pub Uuid);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Account {
    pub id: AccountId,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime,
    pub updated_at: DateTime,
    pub is_api_enabled: bool,
    pub status: AccountStatus,
    pub account_details: Option<AccountDetails>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AccountStatus {
    Initializing,
    Active,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountDetails {
    pub wallet_details: WalletDetails,
    pub balances: Vec<TokenAmount>,
    pub payin_methods: Vec<PayinMethod>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PayinMethod {
    pub status: PayinMethodStatus,
    pub supported_destination_tokens: Vec<DestinationToken>,
    pub payin_rail_details: PayinRailDetails,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PayinMethodStatus {
    Activated,
    Deactivated,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DestinationToken {
    pub fees: Fees,
    pub token: Token,
    pub transaction_minimum: Option<FiatAmount>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Fees {
    pub variable_fee_percentage: f64,
    pub fixed_transaction_fee: Option<FiatAmount>,
    pub developer_fee_percentage: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Token {
    pub symbol: String,
    pub blockchain: Blockchain,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum PayinRailDetails {
    #[serde(rename_all = "camelCase")]
    Usd {
        currency: UsdCurrency,
        payin_rails: Vec<String>,
        bank_beneficiary_name: String,
        bank_beneficiary_address: String,
        bank_name: String,
        bank_address: String,
        bank_routing_number: String,
        bank_account_number: String,
    },
    #[serde(rename_all = "camelCase")]
    Eur {
        currency: EurCurrency,
        payin_rail: EurPayinRail,
        bank_name: String,
        bank_address: String,
        account_holder_name: String,
        iban: String,
        bic: String,
    },
    #[serde(rename_all = "camelCase")]
    Cop {
        currency: CopCurrency,
        payin_rail: CopPayinRail,
    },
    #[serde(rename_all = "camelCase")]
    BlockchainDeposit {
        deposit_token: DepositToken,
        sender_address: Option<String>,
        destination_address: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING-KEBAB-CASE")]
pub enum UsdCurrency {
    Usd,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING-KEBAB-CASE")]
pub enum EurCurrency {
    Eur,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING-KEBAB-CASE")]
pub enum EurPayinRail {
    Sepa,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING-KEBAB-CASE")]
pub enum CopCurrency {
    Cop,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING-KEBAB-CASE")]
pub enum CopPayinRail {
    Pse,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DepositToken {
    #[serde(rename_all = "camelCase")]
    UsdtTron { contract_address: String },
}
