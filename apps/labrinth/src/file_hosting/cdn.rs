use std::{convert::Infallible, sync::Arc};

use actix_http::header::HeaderName;
use actix_utils::future::{Ready, ready};
use actix_web::FromRequest;
use eyre::Result;

use crate::util::env::env_var;

#[derive(Debug, Clone, Copy)]
pub struct UseAltCdn(pub bool);

const HEADER_NAME: HeaderName = HeaderName::from_static("labrinth-alt-cdn");

impl FromRequest for UseAltCdn {
    type Error = Infallible;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(
        req: &actix_web::HttpRequest,
        _payload: &mut actix_http::Payload,
    ) -> Self::Future {
        ready(Ok(Self(use_alt_cdn(req))))
    }
}

fn use_alt_cdn(req: &actix_web::HttpRequest) -> bool {
    let Some(use_alt_cdn) = req.headers().get(HEADER_NAME) else {
        return false;
    };
    use_alt_cdn.as_bytes() == b"true"
}

#[derive(Debug, Clone)]
pub enum CdnChoice {
    Default,
    Alt {
        base_url: Arc<str>,
        alt_url: Arc<str>,
    },
}

impl CdnChoice {
    pub fn transform_file_url(&self, file_url: impl Into<String>) -> String {
        let file_url = file_url.into();
        match self {
            Self::Default => file_url,
            Self::Alt { base_url, alt_url } => {
                file_url.replace(&**base_url, alt_url)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct CdnConfig {
    pub url: Arc<str>,
    pub alt_url: Arc<str>,
}

impl CdnConfig {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            url: Arc::from(env_var("CDN_URL")?),
            alt_url: Arc::from(env_var("CDN_ALT_URL")?),
        })
    }

    pub fn make_choice(&self, use_alt_cdn: bool) -> CdnChoice {
        if use_alt_cdn {
            CdnChoice::Alt {
                base_url: self.url.clone(),
                alt_url: self.alt_url.clone(),
            }
        } else {
            CdnChoice::Default
        }
    }
}
