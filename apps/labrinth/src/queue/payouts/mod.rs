use crate::env::ENV;
use crate::models::payouts::{
    PayoutDecimal, PayoutInterval, PayoutMethod, PayoutMethodType,
    TremendousForexResponse,
};
use crate::routes::ApiError;
use crate::util::error::ApiContext as _;
use crate::util::error::Context;
use arc_swap::ArcSwapOption;
use base64::Engine;
use chrono::{DateTime, Duration, Utc};
use eyre::Result;
use modrinth_util::decimal::Decimal2dp;
use muralpay::FiatAndRailCode;
use reqwest::Method;
use rust_decimal::Decimal;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use tokio::sync::RwLock;
use tracing::warn;

pub mod flow;
pub mod mural;

pub struct PayoutsQueue {
    credential: RwLock<Option<PayPalCredentials>>,
    payout_options: RwLock<Option<PayoutMethods>>,
    pub muralpay: ArcSwapOption<MuralPayConfig>,
}

pub struct MuralPayConfig {
    pub client: muralpay::Client,
    pub source_account_id: muralpay::AccountId,
}

#[derive(Clone, Debug)]
struct PayPalCredentials {
    access_token: String,
    token_type: String,
    expires: DateTime<Utc>,
}

#[derive(Clone)]
struct PayoutMethods {
    options: Vec<PayoutMethod>,
    expires: DateTime<Utc>,
}

impl Default for PayoutsQueue {
    fn default() -> Self {
        Self::new()
    }
}

pub fn create_muralpay_client() -> Result<muralpay::Client> {
    Ok(muralpay::Client::new(
        &ENV.MURALPAY_API_URL,
        ENV.MURALPAY_API_KEY.as_str(),
        ENV.MURALPAY_TRANSFER_API_KEY.as_str(),
    ))
}

pub fn create_muralpay() -> Result<MuralPayConfig> {
    let client = create_muralpay_client()?;
    Ok(MuralPayConfig {
        client,
        source_account_id: ENV.MURALPAY_SOURCE_ACCOUNT_ID,
    })
}

fn create_muralpay_methods() -> Vec<PayoutMethod> {
    let all_countries = rust_iso3166::ALL
        .iter()
        .map(|x| x.alpha2)
        .collect::<Vec<_>>();

    let currencies = vec![
        (
            "blockchain_usdc_polygon",
            "USDC on Polygon",
            all_countries,
            None,
        ),
        ("fiat_mxn", "MXN", vec!["MX"], Some(FiatAndRailCode::Mxn)),
        ("fiat_brl", "BRL", vec!["BR"], Some(FiatAndRailCode::Brl)),
        ("fiat_clp", "CLP", vec!["CL"], Some(FiatAndRailCode::Clp)),
        ("fiat_crc", "CRC", vec!["CR"], Some(FiatAndRailCode::Crc)),
        ("fiat_pen", "PEN", vec!["PE"], Some(FiatAndRailCode::Pen)),
        // ("fiat_dop", "DOP"), // unsupported in API
        // ("fiat_uyu", "UYU"), // unsupported in API
        ("fiat_ars", "ARS", vec!["AR"], Some(FiatAndRailCode::Ars)),
        ("fiat_cop", "COP", vec!["CO"], Some(FiatAndRailCode::Cop)),
        ("fiat_usd", "USD", vec!["US"], Some(FiatAndRailCode::Usd)),
        (
            "fiat_usd-peru",
            "USD Peru",
            vec!["PE"],
            Some(FiatAndRailCode::UsdPeru),
        ),
        // ("fiat_usd-panama", "USD Panama"), // by request
        (
            "fiat_eur",
            "EUR",
            vec![
                "DE", "FR", "IT", "ES", "NL", "BE", "AT", "PT", "FI", "IE",
                "GR", "LU", "CY", "MT", "SK", "SI", "EE", "LV", "LT",
            ],
            Some(FiatAndRailCode::Eur),
        ),
    ];

    currencies
        .into_iter()
        .map(
            |(id, currency, countries, fiat_and_rail_code)| PayoutMethod {
                id: id.to_string(),
                type_: PayoutMethodType::MuralPay,
                name: format!("Mural Pay - {currency}"),
                category: None,
                supported_countries: countries
                    .iter()
                    .map(|s| s.to_string())
                    .collect(),
                image_url: None,
                image_logo_url: None,
                interval: PayoutInterval::Standard {
                    min: if let Some(fiat_and_rail_code) = fiat_and_rail_code {
                        flow::mural::min_usd_fiat(fiat_and_rail_code)
                    } else {
                        flow::mural::MIN_USD_BLOCKCHAIN
                    }
                    .get(),
                    max: flow::mural::MAX_USD.get(),
                },
                currency_code: None,
                exchange_rate: None,
            },
        )
        .collect()
}

// Batches payouts and handles token refresh
impl PayoutsQueue {
    pub fn new() -> Self {
        let muralpay = create_muralpay()
            .inspect_err(|err| {
                warn!("Failed to create Mural Pay client: {err:#?}")
            })
            .ok();

        PayoutsQueue {
            credential: RwLock::new(None),
            payout_options: RwLock::new(None),
            muralpay: ArcSwapOption::from_pointee(muralpay),
        }
    }

    async fn refresh_token(&self) -> Result<PayPalCredentials, ApiError> {
        let mut creds = self.credential.write().await;
        let client = reqwest::Client::new();

        let combined_key =
            format!("{}:{}", ENV.PAYPAL_CLIENT_ID, ENV.PAYPAL_CLIENT_SECRET);
        let formatted_key = format!(
            "Basic {}",
            base64::engine::general_purpose::STANDARD.encode(combined_key)
        );

        let mut form = HashMap::new();
        form.insert("grant_type", "client_credentials");

        #[derive(Deserialize)]
        struct PaypalCredential {
            access_token: String,
            token_type: String,
            expires_in: i64,
        }

        let credential: PaypalCredential = client
            .post(format!("{}oauth2/token", ENV.PAYPAL_API_URL))
            .header("Accept", "application/json")
            .header("Accept-Language", "en_US")
            .header("Authorization", formatted_key)
            .form(&form)
            .send()
            .await
            .map_err(|err| eyre::eyre!(err))
            .wrap_failed_dependency_err(
                "error while authenticating with PayPal".to_string(),
            )?
            .json()
            .await
            .map_err(|err| eyre::eyre!(err))
            .wrap_failed_dependency_err(
                "error while authenticating with PayPal (deser error)"
                    .to_string(),
            )?;

        let new_creds = PayPalCredentials {
            access_token: credential.access_token,
            token_type: credential.token_type,
            expires: Utc::now() + Duration::seconds(credential.expires_in),
        };

        *creds = Some(new_creds.clone());

        Ok(new_creds)
    }

    pub async fn make_paypal_request<T: Serialize, X: DeserializeOwned>(
        &self,
        method: Method,
        path: &str,
        body: Option<T>,
        raw_text: Option<String>,
        no_api_prefix: Option<bool>,
    ) -> Result<X, ApiError> {
        let read = self.credential.read().await;
        let credentials = if let Some(credentials) = read.as_ref() {
            if credentials.expires < Utc::now() {
                drop(read);
                self.refresh_token()
                    .await
                    .map_err(|err| eyre::eyre!(err))
                    .wrap_failed_dependency_err(
                        "error while authenticating with PayPal".to_string(),
                    )?
            } else {
                credentials.clone()
            }
        } else {
            drop(read);
            self.refresh_token()
                .await
                .map_err(|err| eyre::eyre!(err))
                .wrap_failed_dependency_err(
                    "error while authenticating with PayPal".to_string(),
                )?
        };

        let client = reqwest::Client::new();
        let mut request = client
            .request(
                method,
                if no_api_prefix.unwrap_or(false) {
                    path.to_string()
                } else {
                    format!("{}{path}", ENV.PAYPAL_API_URL)
                },
            )
            .header(
                "Authorization",
                format!(
                    "{} {}",
                    credentials.token_type, credentials.access_token
                ),
            );

        if let Some(body) = body {
            request = request.json(&body);
        } else if let Some(body) = raw_text {
            request = request
                .header(reqwest::header::CONTENT_TYPE, "application/json")
                .body(body);
        }

        let resp = request
            .send()
            .await
            .map_err(|err| eyre::eyre!(err))
            .wrap_failed_dependency_err(
                "could not communicate with PayPal".to_string(),
            )?;

        let status = resp.status();

        let value = resp
            .json::<Value>()
            .await
            .map_err(|err| eyre::eyre!(err))
            .wrap_failed_dependency_err(
                "could not retrieve PayPal response body".to_string(),
            )?;

        if !status.is_success() {
            #[derive(Deserialize)]
            struct PayPalError {
                pub name: String,
                pub message: String,
            }

            #[derive(Deserialize)]
            struct PayPalIdentityError {
                pub error: String,
                pub error_description: String,
            }

            if let Ok(mut error) =
                serde_json::from_value::<PayPalError>(value.clone())
            {
                if error.name == "INSUFFICIENT_FUNDS" {
                    error.message = "We're currently transferring funds to our PayPal account. Please try again in a couple days.".to_string();
                }
                return Err(ApiError::FailedDependency(eyre::eyre!(format!(
                    "error name: {}, message: {}",
                    error.name, error.message
                ))));
            }

            if let Ok(error) =
                serde_json::from_value::<PayPalIdentityError>(value)
            {
                return Err(ApiError::FailedDependency(eyre::eyre!(format!(
                    "error name: {}, message: {}",
                    error.error, error.error_description
                ))));
            }

            return Err(ApiError::FailedDependency(eyre::eyre!(
                "could not retrieve PayPal error body",
            )));
        }

        serde_json::from_value(value)
            .wrap_request_err("deserializing JSON data")
    }

    pub async fn make_tremendous_request<T: Serialize, X: DeserializeOwned>(
        &self,
        method: Method,
        path: &str,
        body: Option<T>,
    ) -> Result<X, ApiError> {
        let client = reqwest::Client::new();
        let mut request = client
            .request(method, format!("{}{path}", ENV.TREMENDOUS_API_URL))
            .header(
                "Authorization",
                format!("Bearer {}", ENV.TREMENDOUS_API_KEY),
            );

        if let Some(body) = body {
            request = request.json(&body);
        }

        let resp = request
            .send()
            .await
            .map_err(|err| eyre::eyre!(err))
            .wrap_failed_dependency_err(
                "could not communicate with Tremendous".to_string(),
            )?;

        let status = resp.status();

        let value = resp
            .json::<Value>()
            .await
            .map_err(|err| eyre::eyre!(err))
            .wrap_failed_dependency_err(
                "could not retrieve Tremendous response body".to_string(),
            )?;

        if !status.is_success()
            && let Some(obj) = value.as_object()
        {
            if let Some(array) = obj.get("errors") {
                #[derive(Deserialize)]
                struct TremendousError {
                    message: String,
                    payload: Option<serde_json::Value>,
                }

                let err =
                    serde_json::from_value::<TremendousError>(array.clone())
                        .map_err(|err| eyre::eyre!(err))
                        .wrap_failed_dependency_err(
                            "could not retrieve Tremendous error json body"
                                .to_string(),
                        )?;

                return Err(ApiError::FailedDependency(eyre::eyre!(format!(
                    "Tremendous error: {} ({:?})",
                    err.message, err.payload
                ))));
            }

            return Err(ApiError::FailedDependency(eyre::eyre!(
                "could not retrieve Tremendous error body",
            )));
        }

        serde_json::from_value(value)
            .wrap_request_err("deserializing JSON data")
    }

    pub async fn get_payout_methods(
        &self,
    ) -> Result<Vec<PayoutMethod>, ApiError> {
        async fn refresh_payout_methods(
            queue: &PayoutsQueue,
        ) -> Result<PayoutMethods, ApiError> {
            let mut options = queue.payout_options.write().await;

            let mut methods = Vec::new();

            match get_tremendous_payout_methods(queue).await {
                Ok(mut tremendous_methods) => {
                    methods.append(&mut tremendous_methods);
                }
                Err(err) => {
                    warn!(
                        "Failed to fetch Tremendous payout methods: {err:#?}"
                    );
                }
            }

            {
                let paypal_us = PayoutMethod {
                    id: "paypal_us".to_string(),
                    type_: PayoutMethodType::PayPal,
                    name: "PayPal".to_string(),
                    category: None,
                    supported_countries: vec!["US".to_string()],
                    image_url: None,
                    image_logo_url: None,
                    interval: PayoutInterval::Standard {
                        min: flow::paypal::MIN_USD.get(),
                        max: flow::paypal::MAX_USD.get(),
                    },
                    currency_code: None,
                    exchange_rate: None,
                };

                let mut venmo = paypal_us.clone();
                venmo.id = "venmo".to_string();
                venmo.name = "Venmo".to_string();
                venmo.type_ = PayoutMethodType::Venmo;

                methods.insert(0, paypal_us);
                methods.insert(1, venmo)
            }

            methods.extend(create_muralpay_methods());

            let new_options = PayoutMethods {
                options: methods,
                expires: Utc::now() + Duration::hours(6),
            };

            *options = Some(new_options.clone());

            Ok(new_options)
        }

        let read = self.payout_options.read().await;
        let options = if let Some(options) = read.as_ref() {
            if options.expires < Utc::now() {
                drop(read);
                refresh_payout_methods(self)
                    .await
                    .wrap_api_err("executing `refresh_payout_methods`")?
            } else {
                options.clone()
            }
        } else {
            drop(read);
            refresh_payout_methods(self)
                .await
                .wrap_api_err("executing `refresh_payout_methods`")?
        };

        Ok(options.options)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PayoutFees {
    /// Fee which is taken by the underlying method we're using.
    ///
    /// For example, if a user withdraws $10.00 and the method takes a
    /// 10% cut, then we submit a payout request of $10.00 to the method,
    /// and only $9.00 will be sent to the recipient.
    pub method_fee: Decimal2dp,
    /// Fee which we keep and don't pass to the underlying method.
    ///
    /// For example, if a user withdraws $10.00 and the method takes a
    /// 10% cut, then we submit a payout request of $9.00, and the $1.00 stays
    /// in our account.
    pub platform_fee: Decimal2dp,
    /// How much is 1 USD worth in the target currency?
    pub exchange_rate: Option<Decimal>,
}

impl PayoutFees {
    pub fn total_fee(&self) -> Decimal2dp {
        self.method_fee + self.platform_fee
    }
}

async fn get_tremendous_payout_methods(
    queue: &PayoutsQueue,
) -> Result<Vec<PayoutMethod>> {
    #[derive(Debug, Deserialize)]
    struct Sku {
        min: Decimal,
        max: Decimal,
    }

    #[derive(Deserialize, Eq, PartialEq)]
    #[serde(rename_all = "snake_case")]
    enum ProductImageType {
        Card,
        Logo,
    }

    #[derive(Deserialize)]
    struct ProductImage {
        src: String,
        #[serde(rename = "type")]
        type_: ProductImageType,
    }

    #[derive(Deserialize)]
    struct ProductCountry {
        abbr: String,
    }

    #[derive(Deserialize)]
    struct Product {
        id: String,
        category: String,
        name: String,
        // description: String,
        // disclosure: String,
        skus: Vec<Sku>,
        currency_codes: Vec<String>,
        countries: Vec<ProductCountry>,
        images: Vec<ProductImage>,
    }

    #[derive(Deserialize)]
    struct TremendousResponse {
        products: Vec<Product>,
    }

    let forex: TremendousForexResponse = queue
        .make_tremendous_request(Method::GET, "forex", None::<()>)
        .await
        .wrap_err("failed to fetch Tremendous forex data")?;

    let response = queue
        .make_tremendous_request::<(), TremendousResponse>(
            Method::GET,
            "products",
            None,
        )
        .await
        .wrap_err("failed to fetch Tremendous products data")?;

    let mut methods = Vec::new();

    for product in response.products {
        const BLACKLISTED_IDS: &[&str] = &[
            // typos:off
            // physical visa
            "A2J05SWPI2QG",
            // crypto
            "1UOOSHUUYTAM",
            "5EVJN47HPDFT",
            "NI9M4EVAVGFJ",
            "VLY29QHTMNGT",
            "7XU98H109Y3A",
            "0CGEDFP2UIKV",
            "PDYLQU0K073Y",
            "HCS5Z7O2NV5G",
            "IY1VMST1MOXS",
            "VRPZLJ7HCA8X",
            // bitcard (crypto)
            "GWQQS5RM8IZS",
            "896MYD4SGOGZ",
            "PWLEN1VZGMZA",
            "A2VRM96J5K5W",
            "HV9ICIM3JT7P",
            "K2KLSPVWC2Q4",
            "HRBRQLLTDF95",
            "UUBYLZVK7QAB",
            "BH8W3XEDEOJN",
            "7WGE043X1RYQ",
            "2B13MHUZZVTF",
            "JN6R44P86EYX",
            "DA8H43GU84SO",
            "QK2XAQHSDEH4",
            "J7K1IQFS76DK",
            "NL4JQ2G7UPRZ",
            "OEFTMSBA5ELH",
            "A3CQK6UHNV27",
            // typos:on
        ];
        const SUPPORTED_METHODS: &[&str] = &[
            "merchant_cards",
            "merchant_card",
            "bank",
            "charity",
            "visa",
            "visa_card",
            "paypal",
            "venmo",
        ];

        if !SUPPORTED_METHODS.contains(&&*product.category)
            || BLACKLISTED_IDS.contains(&&*product.id)
        {
            continue;
        };

        let Some(currency) = product.currency_codes.first() else {
            // cards with multiple currencies are not supported
            continue;
        };
        let Some(&usd_to_currency) = forex.forex.get(currency) else {
            warn!("No Tremendous forex data for {currency}");
            continue;
        };

        let method = PayoutMethod {
            id: product.id,
            type_: PayoutMethodType::Tremendous,
            name: product.name.clone(),
            category: Some(product.category.clone()),
            supported_countries: product
                .countries
                .into_iter()
                .map(|x| x.abbr)
                .collect(),
            image_logo_url: product
                .images
                .iter()
                .find(|x| x.type_ == ProductImageType::Logo)
                .map(|x| x.src.clone()),
            image_url: product
                .images
                .into_iter()
                .find(|x| x.type_ == ProductImageType::Card)
                .map(|x| x.src),
            interval: if product.skus.len() > 1 {
                let mut values = product
                    .skus
                    .into_iter()
                    .map(|x| PayoutDecimal(x.min))
                    .collect::<Vec<_>>();
                values.sort_by_key(|a| a.0);

                PayoutInterval::Fixed { values }
            } else if let Some(first) = product.skus.first() {
                PayoutInterval::Standard {
                    min: first.min,
                    max: first.max,
                }
            } else {
                PayoutInterval::Standard {
                    min: Decimal::ZERO,
                    max: Decimal::from(5_000),
                }
            },
            currency_code: Some(currency.clone()),
            exchange_rate: Some(usd_to_currency),
        };

        methods.push(method);
    }

    const UPRANK_IDS: &[&str] =
        &["ET0ZVETV5ILN", "Q24BD9EZ332JT", "UIL1ZYJU5MKN"];
    const DOWNRANK_IDS: &[&str] = &["EIPF8Q00EMM1", "OU2MWXYWPNWQ"];

    methods.sort_by(|a, b| {
        let a_top = UPRANK_IDS.contains(&&*a.id);
        let a_bottom = DOWNRANK_IDS.contains(&&*a.id);
        let b_top = UPRANK_IDS.contains(&&*b.id);
        let b_bottom = DOWNRANK_IDS.contains(&&*b.id);

        match (a_top, a_bottom, b_top, b_bottom) {
            (true, _, true, _) => a.name.cmp(&b.name), // Both in top_priority: sort alphabetically
            (_, true, _, true) => a.name.cmp(&b.name), // Both in bottom_priority: sort alphabetically
            (true, _, _, _) => std::cmp::Ordering::Less, // a in top_priority: a comes first
            (_, _, true, _) => std::cmp::Ordering::Greater, // b in top_priority: b comes first
            (_, true, _, _) => std::cmp::Ordering::Greater, // a in bottom_priority: b comes first
            (_, _, _, true) => std::cmp::Ordering::Less, // b in bottom_priority: a comes first
            (_, _, _, _) => a.name.cmp(&b.name), // Neither in priority: sort alphabetically
        }
    });

    Ok(methods)
}
