use std::str::FromStr;

use chrono::{DateTime, Utc};
use derive_more::{Deref, Display};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{AccountId, Blockchain, FiatAmount, PayoutId, PayoutRequestId, TokenAmount};

#[cfg(feature = "client")]
const _: () = {
    use crate::{Account, MuralError, RequestExt, SearchParams, SearchResponse};

    impl crate::Client {
        pub async fn get_transaction(&self, id: TransactionId) -> Result<Transaction, MuralError> {
            maybe_mock!(self, get_transaction(id));

            self.http_get(|base| format!("{base}/api/transactions/{id}"))
                .send_mural()
                .await
        }

        pub async fn search_transactions(
            &self,
            account_id: AccountId,
            params: Option<SearchParams<AccountId>>,
        ) -> Result<SearchResponse<AccountId, Account>, MuralError> {
            maybe_mock!(self, search_transactions(account_id, params));

            self.http_post(|base| format!("{base}/api/transactions/search/account/{account_id}"))
                .query(&params.map(|p| p.to_query()).unwrap_or_default())
                .send_mural()
                .await
        }
    }
};

#[derive(Debug, Display, Clone, Copy, PartialEq, Eq, Hash, Deref, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[display("{}", _0.hyphenated())]
pub struct TransactionId(pub Uuid);

impl FromStr for TransactionId {
    type Err = <Uuid as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse::<Uuid>().map(Self)
    }
}

impl From<TransactionId> for Uuid {
    fn from(value: TransactionId) -> Self {
        value.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct Transaction {
    pub id: TransactionId,
    pub hash: String,
    pub transaction_execution_date: DateTime<Utc>,
    pub memo: Option<String>,
    pub blockchain: Blockchain,
    pub amount: TokenAmount,
    pub account_id: AccountId,
    // pub counterparty_info,
    pub transaction_details: TransactionDetails,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum TransactionDetails {
    #[serde(rename_all = "camelCase")]
    Payout {
        payout_request_id: PayoutRequestId,
        payout_id: PayoutId,
    },
    #[serde(rename_all = "camelCase")]
    Deposit { details: DepositDetails },
    #[serde(rename_all = "camelCase")]
    ExternalPayout { recipient_wallet_address: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum DepositDetails {
    #[serde(rename_all = "camelCase")]
    Fiat {
        deposit_id: Uuid,
        created_at: DateTime<Utc>,
        sent_fiat_amount: FiatAmount,
        sender_metadata: Option<SenderMetadata>,
        deposit_status_info: DepositStatus,
    },
    #[serde(rename_all = "camelCase")]
    Blockchain {
        sender_address: String,
        blockchain: Blockchain,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum SenderMetadata {
    #[serde(rename_all = "camelCase")]
    Ach {
        ach_routing_number: String,
        sender_name: String,
        description: Option<String>,
        trace_number: String,
    },
    #[serde(rename_all = "camelCase")]
    Wire {
        wire_routing_number: String,
        sender_name: Option<String>,
        bank_name: String,
        bank_beneficiary_name: String,
        imad: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum DepositStatus {
    #[serde(rename_all = "camelCase")]
    AwaitingFunds,
    #[serde(rename_all = "camelCase")]
    Completed {
        initiated_at: DateTime<Utc>,
        completed_at: DateTime<Utc>,
    },
}
