use {
    crate::{Blockchain, FiatAmount, TokenAmount, WalletDetails},
    chrono::{DateTime, Utc},
    derive_more::{Deref, Display},
    rust_decimal::Decimal,
    serde::{Deserialize, Serialize},
    std::str::FromStr,
    uuid::Uuid,
};

#[cfg(feature = "client")]
const _: () = {
    use crate::{MuralError, RequestExt};

    impl crate::Client {
        pub async fn get_all_accounts(&self) -> Result<Vec<Account>, MuralError> {
            maybe_mock!(self, get_all_accounts());

            self.http_get(|base| format!("{base}/api/accounts"))
                .send_mural()
                .await
        }

        pub async fn get_account(&self, id: AccountId) -> Result<Account, MuralError> {
            maybe_mock!(self, get_account(id));

            self.http_get(|base| format!("{base}/api/accounts/{id}"))
                .send_mural()
                .await
        }

        pub async fn create_account(
            &self,
            name: impl AsRef<str>,
            description: Option<impl AsRef<str>>,
        ) -> Result<Account, MuralError> {
            #[derive(Debug, Serialize)]
            #[serde(rename_all = "camelCase")]
            struct Body<'a> {
                name: &'a str,
                description: Option<&'a str>,
            }

            maybe_mock!(
                self,
                create_account(name.as_ref(), description.as_ref().map(AsRef::as_ref))
            );

            let body = Body {
                name: name.as_ref(),
                description: description.as_ref().map(AsRef::as_ref),
            };

            self.http_post(|base| format!("{base}/api/accounts"))
                .json(&body)
                .send_mural()
                .await
        }
    }
};

#[derive(Debug, Display, Clone, Copy, PartialEq, Eq, Hash, Deref, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[display("{}", _0.hyphenated())]
pub struct AccountId(pub Uuid);

impl FromStr for AccountId {
    type Err = <Uuid as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse::<Uuid>().map(Self)
    }
}

impl From<AccountId> for Uuid {
    fn from(value: AccountId) -> Self {
        value.0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct Account {
    pub id: AccountId,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_api_enabled: bool,
    pub status: AccountStatus,
    pub account_details: Option<AccountDetails>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AccountStatus {
    Initializing,
    Active,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct AccountDetails {
    pub wallet_details: WalletDetails,
    pub balances: Vec<TokenAmount>,
    pub payin_methods: Vec<PayinMethod>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct PayinMethod {
    pub status: PayinMethodStatus,
    pub supported_destination_tokens: Vec<DestinationToken>,
    pub payin_rail_details: PayinRailDetails,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PayinMethodStatus {
    Activated,
    Deactivated,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct DestinationToken {
    pub fees: Fees,
    pub token: Token,
    pub transaction_minimum: Option<FiatAmount>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct Fees {
    #[serde(with = "rust_decimal::serde::float")]
    pub variable_fee_percentage: Decimal,
    pub fixed_transaction_fee: Option<FiatAmount>,
    #[serde(with = "rust_decimal::serde::float_option", default)]
    pub developer_fee_percentage: Option<Decimal>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct Token {
    pub symbol: String,
    pub blockchain: Blockchain,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
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
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(rename_all = "SCREAMING-KEBAB-CASE")]
pub enum UsdCurrency {
    Usd,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(rename_all = "SCREAMING-KEBAB-CASE")]
pub enum EurCurrency {
    Eur,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(rename_all = "SCREAMING-KEBAB-CASE")]
pub enum EurPayinRail {
    Sepa,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(rename_all = "SCREAMING-KEBAB-CASE")]
pub enum CopCurrency {
    Cop,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(rename_all = "SCREAMING-KEBAB-CASE")]
pub enum CopPayinRail {
    Pse,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DepositToken {
    #[serde(rename_all = "camelCase")]
    UsdtTron { contract_address: String },
}
