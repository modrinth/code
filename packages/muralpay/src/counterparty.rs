use std::str::FromStr;

use derive_more::{Deref, Display};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{MuralError, SearchRequest, SearchResponse};

impl MuralPay {
    pub fn search_counterparties(
        &self,
        req: SearchRequest,
    ) -> Result<SearchResponse<CounterpartyId, Counterparty>, MuralError> {
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
pub struct CounterpartyId(pub Uuid);

impl FromStr for CounterpartyId {
    type Err = <Uuid as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse::<Uuid>().map(Self)
    }
}

pub struct Counterparty {}
