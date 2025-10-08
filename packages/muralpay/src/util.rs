use reqwest::{IntoUrl, RequestBuilder};
use secrecy::ExposeSecret;
use serde::de::DeserializeOwned;

use crate::{ApiError, MuralError, MuralPay, TransferError};

impl MuralPay {
    pub(crate) fn http_get<U: IntoUrl>(
        &self,
        make_url: impl FnOnce(&str) -> U,
    ) -> RequestBuilder {
        self.http
            .get(make_url(&self.api_url))
            .bearer_auth(self.api_key.expose_secret())
            .header("accept", "application/json")
            .header("content-type", "application/json")
    }

    pub(crate) fn http_post<U: IntoUrl>(
        &self,
        make_url: impl FnOnce(&str) -> U,
    ) -> RequestBuilder {
        self.http
            .post(make_url(&self.api_url))
            .bearer_auth(self.api_key.expose_secret())
            .header("accept", "application/json")
            .header("content-type", "application/json")
    }
}

pub trait RequestExt: Sized {
    fn transfer_auth(self, client: &MuralPay) -> Result<Self, TransferError>;

    async fn send_mural<T: DeserializeOwned>(self) -> crate::Result<T>;
}

const HEADER_TRANSFER_API_KEY: &str = "transfer-api-key";

impl RequestExt for reqwest::RequestBuilder {
    fn transfer_auth(self, client: &MuralPay) -> Result<Self, TransferError> {
        let transfer_api_key = client
            .transfer_api_key
            .as_ref()
            .ok_or(TransferError::NoTransferKey)?;

        Ok(self
            .header(HEADER_TRANSFER_API_KEY, transfer_api_key.expose_secret()))
    }

    async fn send_mural<T: DeserializeOwned>(self) -> crate::Result<T> {
        let resp = self.send().await?;
        let status = resp.status();
        if status.is_client_error() || status.is_server_error() {
            let json = resp.bytes().await?;
            let err = serde_json::from_slice::<ApiError>(&json)
                .map_err(|source| MuralError::DecodeError { source, json })?;
            Err(MuralError::Api(err))
        } else {
            let json = resp.bytes().await?;
            let t = serde_json::from_slice::<T>(&json)
                .map_err(|source| MuralError::Decode { source, json })?;
            Ok(t)
        }
    }
}
