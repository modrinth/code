use {
    crate::{
        ArsSymbol, BobSymbol, BrlSymbol, ClpSymbol, CopSymbol, CounterpartyId, CrcSymbol,
        DocumentType, EurSymbol, FiatAccountType, MxnSymbol, PenSymbol, UsdSymbol, WalletDetails,
        ZarSymbol,
    },
    chrono::{DateTime, Utc},
    derive_more::{Deref, Display, Error},
    serde::{Deserialize, Serialize},
    serde_with::DeserializeFromStr,
    std::str::FromStr,
    uuid::Uuid,
};

#[cfg(feature = "client")]
const _: () = {
    use crate::{MuralError, RequestExt, SearchParams, SearchResponse};

    impl crate::Client {
        pub async fn search_payout_methods(
            &self,
            counterparty_id: CounterpartyId,
            params: Option<SearchParams<PayoutMethodId>>,
        ) -> Result<SearchResponse<PayoutMethodId, PayoutMethod>, MuralError> {
            maybe_mock!(self, search_payout_methods(counterparty_id, params));

            self.http_post(|base| {
                format!("{base}/api/counterparties/{counterparty_id}/payout-methods/search")
            })
            .query(&params.map(|p| p.to_query()).unwrap_or_default())
            .send_mural()
            .await
        }

        pub async fn get_payout_method(
            &self,
            counterparty_id: CounterpartyId,
            payout_method_id: PayoutMethodId,
        ) -> Result<PayoutMethod, MuralError> {
            maybe_mock!(self, get_payout_method(counterparty_id, payout_method_id));

            self.http_get(|base| {
                format!(
                    "{base}/api/counterparties/{counterparty_id}/payout-methods/{payout_method_id}"
                )
            })
            .send_mural()
            .await
        }

        pub async fn create_payout_method(
            &self,
            counterparty_id: CounterpartyId,
            alias: impl AsRef<str>,
            payout_method: &PayoutMethodDetails,
        ) -> Result<PayoutMethod, MuralError> {
            #[derive(Debug, Serialize)]
            #[serde(rename_all = "camelCase")]
            struct Body<'a> {
                alias: &'a str,
                payout_method: &'a PayoutMethodDetails,
            }

            maybe_mock!(
                self,
                create_payout_method(counterparty_id, alias.as_ref(), payout_method)
            );

            let body = Body {
                alias: alias.as_ref(),
                payout_method,
            };

            self.http_post(|base| {
                format!("{base}/api/counterparties/{counterparty_id}/payout-methods")
            })
            .json(&body)
            .send_mural()
            .await
        }

        pub async fn delete_payout_method(
            &self,
            counterparty_id: CounterpartyId,
            payout_method_id: PayoutMethodId,
        ) -> Result<(), MuralError> {
            maybe_mock!(
                self,
                delete_payout_method(counterparty_id, payout_method_id)
            );

            self.http_delete(|base| {
                format!(
                    "{base}/api/counterparties/{counterparty_id}/payout-methods/{payout_method_id}"
                )
            })
            .send_mural()
            .await
        }
    }
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PayoutMethodDocumentType {
    NationalId,
    Passport,
    ResidentId,
    Ruc,
    TaxId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PayoutMethodPixAccountType {
    Phone,
    Email,
    Document,
    BankAccount,
}

#[derive(Debug, Display, Clone, Copy, PartialEq, Eq, Hash, Deref, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[display("{}", _0.hyphenated())]
pub struct PayoutMethodId(pub Uuid);

impl FromStr for PayoutMethodId {
    type Err = <Uuid as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse::<Uuid>().map(Self)
    }
}

#[derive(Debug, Clone, Serialize, DeserializeFromStr)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct TruncatedString(String);

const TRUNCATED_LEN: usize = 4;

#[derive(Debug, Display, Error)]
#[display("expected {TRUNCATED_LEN} characters, got {num_chars}")]
pub struct InvalidTruncated {
    pub num_chars: usize,
}

impl FromStr for TruncatedString {
    type Err = InvalidTruncated;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let num_chars = s.chars().count();
        if num_chars == TRUNCATED_LEN {
            Ok(Self(s.to_string()))
        } else {
            Err(InvalidTruncated { num_chars })
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct PayoutMethod {
    pub id: PayoutMethodId,
    pub created_at: DateTime<Utc>,
    pub counterparty_id: CounterpartyId,
    pub alias: String,
    pub payout_method: PayoutMethodDetails,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum PayoutMethodDetails {
    #[serde(rename_all = "camelCase")]
    Usd { details: UsdPayoutDetails },
    #[serde(rename_all = "camelCase")]
    Ars { details: ArsPayoutDetails },
    #[serde(rename_all = "camelCase")]
    Brl { details: BrlPayoutDetails },
    #[serde(rename_all = "camelCase")]
    Cop { details: CopPayoutDetails },
    #[serde(rename_all = "camelCase")]
    Eur { details: EurPayoutDetails },
    #[serde(rename_all = "camelCase")]
    Mxn { details: MxnPayoutDetails },
    #[serde(rename_all = "camelCase")]
    Clp { details: ClpPayoutDetails },
    #[serde(rename_all = "camelCase")]
    Pen { details: PenPayoutDetails },
    #[serde(rename_all = "camelCase")]
    Bob { details: BobPayoutDetails },
    #[serde(rename_all = "camelCase")]
    Crc { details: CrcPayoutDetails },
    #[serde(rename_all = "camelCase")]
    Zar { details: ZarPayoutDetails },
    #[serde(rename_all = "camelCase")]
    BlockchainWallet { details: WalletDetails },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum UsdPayoutDetails {
    #[serde(rename_all = "camelCase")]
    UsdDomestic {
        symbol: UsdSymbol,
        account_type: FiatAccountType,
        transfer_type: UsdTransferType,
        bank_name: String,
        bank_account_number_truncated: TruncatedString,
        bank_routing_number_truncated: TruncatedString,
    },
    #[serde(rename_all = "camelCase")]
    UsdPeru {
        symbol: UsdSymbol,
        account_type: FiatAccountType,
        document_type: DocumentType,
        bank_name: String,
        bank_account_number_truncated: TruncatedString,
        document_number_truncated: TruncatedString,
    },
    #[serde(rename_all = "camelCase")]
    UsdChina {
        symbol: UsdSymbol,
        account_type: FiatAccountType,
        document_type: DocumentType,
        bank_name: String,
        bank_account_number_truncated: TruncatedString,
        document_number_truncated: TruncatedString,
        swift_bic_truncated: TruncatedString,
        phone_number_truncated: TruncatedString,
    },
    #[serde(rename_all = "camelCase")]
    UsdPanama {
        symbol: UsdSymbol,
        account_type: FiatAccountType,
        document_type: DocumentType,
        bank_name: String,
        bank_account_number_truncated: TruncatedString,
        document_number_truncated: TruncatedString,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum UsdTransferType {
    Ach,
    Wire,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum ArsPayoutDetails {
    #[serde(rename_all = "camelCase")]
    ArsAlias {
        symbol: ArsSymbol,
        bank_name: String,
        alias_truncated: TruncatedString,
        document_number_truncated: TruncatedString,
    },
    #[serde(rename_all = "camelCase")]
    ArsAccountNumber {
        symbol: ArsSymbol,
        bank_account_number_type: ArsBankAccountNumberType,
        bank_name: String,
        bank_account_number_truncated: TruncatedString,
        document_number_truncated: TruncatedString,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ArsBankAccountNumberType {
    Cvu,
    Cbu,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum BrlPayoutDetails {
    #[serde(rename_all = "camelCase")]
    PixPhone {
        symbol: BrlSymbol,
        full_legal_name: String,
        bank_name: String,
        phone_number_truncated: TruncatedString,
        document_number_truncated: TruncatedString,
    },
    #[serde(rename_all = "camelCase")]
    PixEmail {
        symbol: BrlSymbol,
        full_legal_name: String,
        bank_name: String,
        email_truncated: TruncatedString,
        document_number_truncated: TruncatedString,
    },
    #[serde(rename_all = "camelCase")]
    PixDocument {
        symbol: BrlSymbol,
        full_legal_name: String,
        bank_name: String,
        document_number_truncated: TruncatedString,
    },
    #[serde(rename_all = "camelCase")]
    PixBankAccount {
        symbol: BrlSymbol,
        full_legal_name: String,
        bank_name: String,
        bank_account_number_truncated: TruncatedString,
        document_number_truncated: TruncatedString,
    },
    #[serde(rename_all = "camelCase")]
    Wire {
        symbol: BrlSymbol,
        account_type: FiatAccountType,
        full_legal_name: String,
        bank_name: String,
        account_number_truncated: TruncatedString,
        bank_branch_truncated: TruncatedString,
        document_number_truncated: TruncatedString,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum CopPayoutDetails {
    #[serde(rename_all = "camelCase")]
    CopDomestic {
        symbol: CopSymbol,
        account_type: FiatAccountType,
        document_type: DocumentType,
        bank_name: String,
        phone_number_truncated: TruncatedString,
        document_number_truncated: TruncatedString,
        bank_account_number_truncated: TruncatedString,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum EurPayoutDetails {
    #[serde(rename_all = "camelCase")]
    EurSepa {
        symbol: EurSymbol,
        country: String,
        bank_name: String,
        iban_truncated: TruncatedString,
        swift_bic_truncated: TruncatedString,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum MxnPayoutDetails {
    #[serde(rename_all = "camelCase")]
    MxnDomestic {
        symbol: MxnSymbol,
        bank_name: String,
        bank_account_number_truncated: TruncatedString,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum ClpPayoutDetails {
    #[serde(rename_all = "camelCase")]
    ClpDomestic {
        clp: ClpSymbol,
        account_type: FiatAccountType,
        document_type: DocumentType,
        bank_name: String,
        bank_account_number_truncated: TruncatedString,
        document_number_truncated: TruncatedString,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum PenPayoutDetails {
    #[serde(rename_all = "camelCase")]
    PenDomestic {
        symbol: PenSymbol,
        document_type: DocumentType,
        account_type: FiatAccountType,
        bank_name: String,
        document_number_truncated: TruncatedString,
        bank_account_number_truncated: TruncatedString,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum BobPayoutDetails {
    #[serde(rename_all = "camelCase")]
    BobDomestic {
        symbol: BobSymbol,
        document_type: DocumentType,
        bank_name: String,
        bank_account_number_truncated: TruncatedString,
        document_number_truncated: TruncatedString,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum CrcPayoutDetails {
    #[serde(rename_all = "camelCase")]
    CrcDomestic {
        symbol: CrcSymbol,
        document_type: DocumentType,
        bank_name: String,
        iban_truncated: TruncatedString,
        document_number_truncated: TruncatedString,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum ZarPayoutDetails {
    #[serde(rename_all = "camelCase")]
    ZarDomestic {
        symbol: ZarSymbol,
        account_type: FiatAccountType,
        bank_name: String,
        bank_account_number_truncated: TruncatedString,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct CreatePayoutMethod {
    pub alias: String,
    pub payout_method: PayoutMethodDetails,
}
