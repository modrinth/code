use crate::models::payouts::{
    PayoutDecimal, PayoutInterval, PayoutMethod, PayoutMethodFee,
    PayoutMethodType,
};
use crate::models::projects::MonetizationStatus;
use crate::routes::ApiError;
use base64::Engine;
use chrono::{DateTime, Datelike, Duration, NaiveTime, TimeZone, Utc};
use dashmap::DashMap;
use futures::TryStreamExt;
use reqwest::Method;
use rust_decimal::Decimal;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::PgPool;
use sqlx::postgres::PgQueryResult;
use std::collections::HashMap;
use tokio::sync::RwLock;

pub struct PayoutsQueue {
    credential: RwLock<Option<PayPalCredentials>>,
    payout_options: RwLock<Option<PayoutMethods>>,
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

#[derive(Serialize)]
pub struct AccountBalance {
    pub available: Decimal,
    pub pending: Decimal,
}

impl Default for PayoutsQueue {
    fn default() -> Self {
        Self::new()
    }
}
// Batches payouts and handles token refresh
impl PayoutsQueue {
    pub fn new() -> Self {
        PayoutsQueue {
            credential: RwLock::new(None),
            payout_options: RwLock::new(None),
        }
    }

    async fn refresh_token(&self) -> Result<PayPalCredentials, ApiError> {
        let mut creds = self.credential.write().await;
        let client = reqwest::Client::new();

        let combined_key = format!(
            "{}:{}",
            dotenvy::var("PAYPAL_CLIENT_ID")?,
            dotenvy::var("PAYPAL_CLIENT_SECRET")?
        );
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
            .post(format!("{}oauth2/token", dotenvy::var("PAYPAL_API_URL")?))
            .header("Accept", "application/json")
            .header("Accept-Language", "en_US")
            .header("Authorization", formatted_key)
            .form(&form)
            .send()
            .await
            .map_err(|_| {
                ApiError::Payments(
                    "Error while authenticating with PayPal".to_string(),
                )
            })?
            .json()
            .await
            .map_err(|_| {
                ApiError::Payments(
                    "Error while authenticating with PayPal (deser error)"
                        .to_string(),
                )
            })?;

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
                self.refresh_token().await.map_err(|_| {
                    ApiError::Payments(
                        "Error while authenticating with PayPal".to_string(),
                    )
                })?
            } else {
                credentials.clone()
            }
        } else {
            drop(read);
            self.refresh_token().await.map_err(|_| {
                ApiError::Payments(
                    "Error while authenticating with PayPal".to_string(),
                )
            })?
        };

        let client = reqwest::Client::new();
        let mut request = client
            .request(
                method,
                if no_api_prefix.unwrap_or(false) {
                    path.to_string()
                } else {
                    format!("{}{path}", dotenvy::var("PAYPAL_API_URL")?)
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

        let resp = request.send().await.map_err(|_| {
            ApiError::Payments("could not communicate with PayPal".to_string())
        })?;

        let status = resp.status();

        let value = resp.json::<Value>().await.map_err(|_| {
            ApiError::Payments(
                "could not retrieve PayPal response body".to_string(),
            )
        })?;

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
                return Err(ApiError::Payments(format!(
                    "error name: {}, message: {}",
                    error.name, error.message
                )));
            }

            if let Ok(error) =
                serde_json::from_value::<PayPalIdentityError>(value)
            {
                return Err(ApiError::Payments(format!(
                    "error name: {}, message: {}",
                    error.error, error.error_description
                )));
            }

            return Err(ApiError::Payments(
                "could not retrieve PayPal error body".to_string(),
            ));
        }

        Ok(serde_json::from_value(value)?)
    }

    pub async fn make_tremendous_request<T: Serialize, X: DeserializeOwned>(
        &self,
        method: Method,
        path: &str,
        body: Option<T>,
    ) -> Result<X, ApiError> {
        let client = reqwest::Client::new();
        let mut request = client
            .request(
                method,
                format!("{}{path}", dotenvy::var("TREMENDOUS_API_URL")?),
            )
            .header(
                "Authorization",
                format!("Bearer {}", dotenvy::var("TREMENDOUS_API_KEY")?),
            );

        if let Some(body) = body {
            request = request.json(&body);
        }

        let resp = request.send().await.map_err(|_| {
            ApiError::Payments(
                "could not communicate with Tremendous".to_string(),
            )
        })?;

        let status = resp.status();

        let value = resp.json::<Value>().await.map_err(|_| {
            ApiError::Payments(
                "could not retrieve Tremendous response body".to_string(),
            )
        })?;

        if !status.is_success() {
            if let Some(obj) = value.as_object() {
                if let Some(array) = obj.get("errors") {
                    #[derive(Deserialize)]
                    struct TremendousError {
                        message: String,
                    }

                    let err = serde_json::from_value::<TremendousError>(
                        array.clone(),
                    )
                    .map_err(|_| {
                        ApiError::Payments(
                            "could not retrieve Tremendous error json body"
                                .to_string(),
                        )
                    })?;

                    return Err(ApiError::Payments(err.message));
                }

                return Err(ApiError::Payments(
                    "could not retrieve Tremendous error body".to_string(),
                ));
            }
        }

        Ok(serde_json::from_value(value)?)
    }

    pub async fn get_payout_methods(
        &self,
    ) -> Result<Vec<PayoutMethod>, ApiError> {
        async fn refresh_payout_methods(
            queue: &PayoutsQueue,
        ) -> Result<PayoutMethods, ApiError> {
            let mut options = queue.payout_options.write().await;

            let mut methods = Vec::new();

            #[derive(Deserialize)]
            pub struct Sku {
                pub min: Decimal,
                pub max: Decimal,
            }

            #[derive(Deserialize, Eq, PartialEq)]
            #[serde(rename_all = "snake_case")]
            pub enum ProductImageType {
                Card,
                Logo,
            }

            #[derive(Deserialize)]
            pub struct ProductImage {
                pub src: String,
                #[serde(rename = "type")]
                pub type_: ProductImageType,
            }

            #[derive(Deserialize)]
            pub struct ProductCountry {
                pub abbr: String,
            }

            #[derive(Deserialize)]
            pub struct Product {
                pub id: String,
                pub category: String,
                pub name: String,
                // pub description: String,
                // pub disclosure: String,
                pub skus: Vec<Sku>,
                pub currency_codes: Vec<String>,
                pub countries: Vec<ProductCountry>,
                pub images: Vec<ProductImage>,
            }

            #[derive(Deserialize)]
            pub struct TremendousResponse {
                pub products: Vec<Product>,
            }

            let response = queue
                .make_tremendous_request::<(), TremendousResponse>(
                    Method::GET,
                    "products",
                    None,
                )
                .await?;

            for product in response.products {
                const BLACKLISTED_IDS: &[&str] = &[
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
                ];
                const SUPPORTED_METHODS: &[&str] = &[
                    "merchant_cards",
                    "merchant_card",
                    "visa",
                    "bank",
                    "ach",
                    "visa_card",
                    "charity",
                ];

                if !SUPPORTED_METHODS.contains(&&*product.category)
                    || BLACKLISTED_IDS.contains(&&*product.id)
                {
                    continue;
                };

                let method = PayoutMethod {
                    id: product.id,
                    type_: PayoutMethodType::Tremendous,
                    name: product.name.clone(),
                    supported_countries: product
                        .countries
                        .into_iter()
                        .map(|x| x.abbr)
                        .collect(),
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
                        values.sort_by(|a, b| a.0.cmp(&b.0));

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
                    fee: if product.category == "ach" {
                        PayoutMethodFee {
                            percentage: Decimal::from(4) / Decimal::from(100),
                            min: Decimal::from(1) / Decimal::from(4),
                            max: None,
                        }
                    } else {
                        PayoutMethodFee {
                            percentage: Decimal::default(),
                            min: Decimal::default(),
                            max: None,
                        }
                    },
                };

                // we do not support interval gift cards with non US based currencies since we cannot do currency conversions properly
                if let PayoutInterval::Fixed { .. } = method.interval {
                    if !product.currency_codes.contains(&"USD".to_string()) {
                        continue;
                    }
                }

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

            {
                let paypal_us = PayoutMethod {
                    id: "paypal_us".to_string(),
                    type_: PayoutMethodType::PayPal,
                    name: "PayPal".to_string(),
                    supported_countries: vec!["US".to_string()],
                    image_url: None,
                    interval: PayoutInterval::Standard {
                        min: Decimal::from(1) / Decimal::from(4),
                        max: Decimal::from(100_000),
                    },
                    fee: PayoutMethodFee {
                        percentage: Decimal::from(2) / Decimal::from(100),
                        min: Decimal::from(1) / Decimal::from(4),
                        max: Some(Decimal::from(1)),
                    },
                };

                let mut venmo = paypal_us.clone();
                venmo.id = "venmo".to_string();
                venmo.name = "Venmo".to_string();
                venmo.type_ = PayoutMethodType::Venmo;

                methods.insert(0, paypal_us);
                methods.insert(1, venmo)
            }

            methods.insert(
                2,
                PayoutMethod {
                    id: "paypal_in".to_string(),
                    type_: PayoutMethodType::PayPal,
                    name: "PayPal".to_string(),
                    supported_countries: rust_iso3166::ALL
                        .iter()
                        .filter(|x| x.alpha2 != "US")
                        .map(|x| x.alpha2.to_string())
                        .collect(),
                    image_url: None,
                    interval: PayoutInterval::Standard {
                        min: Decimal::from(1) / Decimal::from(4),
                        max: Decimal::from(100_000),
                    },
                    fee: PayoutMethodFee {
                        percentage: Decimal::from(2) / Decimal::from(100),
                        min: Decimal::ZERO,
                        max: Some(Decimal::from(20)),
                    },
                },
            );

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
                refresh_payout_methods(self).await?
            } else {
                options.clone()
            }
        } else {
            drop(read);
            refresh_payout_methods(self).await?
        };

        Ok(options.options)
    }

    pub async fn get_brex_balance() -> Result<Option<AccountBalance>, ApiError>
    {
        #[derive(Deserialize)]
        struct BrexBalance {
            pub amount: i64,
            // pub currency: String,
        }

        #[derive(Deserialize)]
        struct BrexAccount {
            pub current_balance: BrexBalance,
            pub available_balance: BrexBalance,
        }

        #[derive(Deserialize)]
        struct BrexResponse {
            pub items: Vec<BrexAccount>,
        }

        let client = reqwest::Client::new();
        let res = client
            .get(format!("{}accounts/cash", dotenvy::var("BREX_API_URL")?))
            .bearer_auth(&dotenvy::var("BREX_API_KEY")?)
            .send()
            .await?
            .json::<BrexResponse>()
            .await?;

        Ok(Some(AccountBalance {
            available: Decimal::from(
                res.items
                    .iter()
                    .map(|x| x.available_balance.amount)
                    .sum::<i64>(),
            ) / Decimal::from(100),
            pending: Decimal::from(
                res.items
                    .iter()
                    .map(|x| {
                        x.current_balance.amount - x.available_balance.amount
                    })
                    .sum::<i64>(),
            ) / Decimal::from(100),
        }))
    }

    pub async fn get_paypal_balance() -> Result<Option<AccountBalance>, ApiError>
    {
        let api_username = dotenvy::var("PAYPAL_NVP_USERNAME")?;
        let api_password = dotenvy::var("PAYPAL_NVP_PASSWORD")?;
        let api_signature = dotenvy::var("PAYPAL_NVP_SIGNATURE")?;

        let mut params = HashMap::new();
        params.insert("METHOD", "GetBalance");
        params.insert("VERSION", "204");
        params.insert("USER", &api_username);
        params.insert("PWD", &api_password);
        params.insert("SIGNATURE", &api_signature);
        params.insert("RETURNALLCURRENCIES", "1");

        let endpoint = "https://api-3t.paypal.com/nvp";

        let client = reqwest::Client::new();
        let response = client.post(endpoint).form(&params).send().await?;

        let text = response.text().await?;
        let body = urlencoding::decode(&text).unwrap_or_default();

        let mut key_value_map = HashMap::new();

        for pair in body.split('&') {
            let mut iter = pair.splitn(2, '=');
            if let (Some(key), Some(value)) = (iter.next(), iter.next()) {
                key_value_map.insert(key.to_string(), value.to_string());
            }
        }

        if let Some(amount) = key_value_map
            .get("L_AMT0")
            .and_then(|x| Decimal::from_str_exact(x).ok())
        {
            Ok(Some(AccountBalance {
                available: amount,
                pending: Decimal::ZERO,
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn get_tremendous_balance(
        &self,
    ) -> Result<Option<AccountBalance>, ApiError> {
        #[derive(Deserialize)]
        struct FundingSourceMeta {
            available_cents: u64,
            pending_cents: u64,
        }

        #[derive(Deserialize)]
        struct FundingSource {
            method: String,
            meta: FundingSourceMeta,
        }

        #[derive(Deserialize)]
        struct FundingSourceRequest {
            pub funding_sources: Vec<FundingSource>,
        }

        let val = self
            .make_tremendous_request::<(), FundingSourceRequest>(
                Method::GET,
                "funding_sources",
                None,
            )
            .await?;

        Ok(val
            .funding_sources
            .into_iter()
            .find(|x| x.method == "balance")
            .map(|x| AccountBalance {
                available: Decimal::from(x.meta.available_cents)
                    / Decimal::from(100),
                pending: Decimal::from(x.meta.pending_cents)
                    / Decimal::from(100),
            }))
    }
}

#[derive(Deserialize)]
pub struct AditudePoints {
    #[serde(rename = "pointsList")]
    pub points_list: Vec<AditudePoint>,
}

#[derive(Deserialize)]
pub struct AditudePoint {
    pub metric: AditudeMetric,
    pub time: AditudeTime,
}

#[derive(Deserialize)]
pub struct AditudeMetric {
    pub revenue: Option<Decimal>,
    pub impressions: Option<u128>,
    pub cpm: Option<Decimal>,
}

#[derive(Deserialize)]
pub struct AditudeTime {
    pub seconds: u64,
}

pub async fn make_aditude_request(
    metrics: &[&str],
    range: &str,
    interval: &str,
) -> Result<Vec<AditudePoints>, ApiError> {
    let request = reqwest::Client::new()
        .post("https://cloud.aditude.io/api/public/insights/metrics")
        .bearer_auth(&dotenvy::var("ADITUDE_API_KEY")?)
        .json(&serde_json::json!({
            "metrics": metrics,
            "range": range,
            "interval": interval
        }))
        .send()
        .await?
        .error_for_status()?;

    let text = request.text().await?;

    let json: Vec<AditudePoints> = serde_json::from_str(&text)?;

    Ok(json)
}

pub async fn process_payout(
    pool: &PgPool,
    client: &clickhouse::Client,
) -> Result<(), ApiError> {
    sqlx::query!(
        "
        UPDATE payouts
        SET status = $1
        WHERE status = $2 AND created < NOW() - INTERVAL '30 days'
        ",
        crate::models::payouts::PayoutStatus::Failed.as_str(),
        crate::models::payouts::PayoutStatus::InTransit.as_str(),
    )
    .execute(pool)
    .await?;

    let start: DateTime<Utc> = DateTime::from_naive_utc_and_offset(
        (Utc::now() - Duration::days(1))
            .date_naive()
            .and_hms_nano_opt(0, 0, 0, 0)
            .unwrap_or_default(),
        Utc,
    );

    let results = sqlx::query!(
        "SELECT EXISTS(SELECT 1 FROM payouts_values WHERE created = $1)",
        start,
    )
    .fetch_one(pool)
    .await?;

    if results.exists.unwrap_or(false) {
        return Ok(());
    }

    let end = start + Duration::days(1);
    #[derive(Deserialize, clickhouse::Row)]
    struct ProjectMultiplier {
        pub page_views: u64,
        pub project_id: u64,
    }

    let (views_values, views_sum, downloads_values, downloads_sum) = futures::future::try_join4(
        client
            .query(
                r#"
                SELECT COUNT(1) page_views, project_id
                FROM views
                WHERE (recorded BETWEEN ? AND ?) AND (project_id != 0) AND (monetized = TRUE)
                GROUP BY project_id
                ORDER BY page_views DESC
                "#,
            )
            .bind(start.timestamp())
            .bind(end.timestamp())
            .fetch_all::<ProjectMultiplier>(),
        client
            .query("SELECT COUNT(1) FROM views WHERE (recorded BETWEEN ? AND ?) AND (project_id != 0) AND (monetized = TRUE)")
            .bind(start.timestamp())
            .bind(end.timestamp())
            .fetch_one::<u64>(),
        client
            .query(
                r#"
                SELECT COUNT(1) page_views, project_id
                FROM downloads
                WHERE (recorded BETWEEN ? AND ?) AND (user_id != 0)
                GROUP BY project_id
                ORDER BY page_views DESC
                "#,
            )
            .bind(start.timestamp())
            .bind(end.timestamp())
            .fetch_all::<ProjectMultiplier>(),
        client
            .query("SELECT COUNT(1) FROM downloads WHERE (recorded BETWEEN ? AND ?) AND (user_id != 0)")
            .bind(start.timestamp())
            .bind(end.timestamp())
            .fetch_one::<u64>(),
    )
        .await?;

    let mut transaction = pool.begin().await?;

    struct PayoutMultipliers {
        sum: u64,
        values: HashMap<u64, u64>,
    }

    let mut views_values = views_values
        .into_iter()
        .map(|x| (x.project_id, x.page_views))
        .collect::<HashMap<u64, u64>>();
    let downloads_values = downloads_values
        .into_iter()
        .map(|x| (x.project_id, x.page_views))
        .collect::<HashMap<u64, u64>>();

    for (key, value) in &downloads_values {
        let counter = views_values.entry(*key).or_insert(0);
        *counter += *value;
    }

    let multipliers: PayoutMultipliers = PayoutMultipliers {
        sum: downloads_sum + views_sum,
        values: views_values,
    };

    struct Project {
        // user_id, payouts_split
        team_members: Vec<(i64, Decimal)>,
    }

    let mut projects_map: HashMap<i64, Project> = HashMap::new();

    let project_ids = multipliers
        .values
        .keys()
        .map(|x| *x as i64)
        .collect::<Vec<i64>>();

    let project_org_members = sqlx::query!(
        "
        SELECT m.id id, tm.user_id user_id, tm.payouts_split payouts_split
        FROM mods m
        INNER JOIN organizations o ON m.organization_id = o.id
        INNER JOIN team_members tm on o.team_id = tm.team_id AND tm.accepted = TRUE
        WHERE m.id = ANY($1) AND m.monetization_status = $2 AND m.status = ANY($3) AND m.organization_id IS NOT NULL
        ",
        &project_ids,
        MonetizationStatus::Monetized.as_str(),
        &*crate::models::projects::ProjectStatus::iterator()
            .filter(|x| !x.is_hidden())
            .map(|x| x.to_string())
            .collect::<Vec<String>>(),
    )
    .fetch(&mut *transaction)
    .try_fold(DashMap::new(), |acc: DashMap<i64, HashMap<i64, Decimal>>, r| {
        acc.entry(r.id)
            .or_default()
            .insert(r.user_id, r.payouts_split);
        async move { Ok(acc) }
    })
    .await?;

    let project_team_members = sqlx::query!(
        "
        SELECT m.id id, tm.user_id user_id, tm.payouts_split payouts_split
        FROM mods m
        INNER JOIN team_members tm on m.team_id = tm.team_id AND tm.accepted = TRUE
        WHERE m.id = ANY($1) AND m.monetization_status = $2 AND m.status = ANY($3)
        ",
        &project_ids,
        MonetizationStatus::Monetized.as_str(),
        &*crate::models::projects::ProjectStatus::iterator()
            .filter(|x| !x.is_hidden())
            .map(|x| x.to_string())
            .collect::<Vec<String>>(),
    )
    .fetch(&mut *transaction)
    .try_fold(
        DashMap::new(),
        |acc: DashMap<i64, HashMap<i64, Decimal>>, r| {
            acc.entry(r.id)
                .or_default()
                .insert(r.user_id, r.payouts_split);
            async move { Ok(acc) }
        },
    )
    .await?;

    for project_id in project_ids {
        let team_members: HashMap<i64, Decimal> = project_team_members
            .remove(&project_id)
            .unwrap_or((0, HashMap::new()))
            .1;
        let org_team_members: HashMap<i64, Decimal> = project_org_members
            .remove(&project_id)
            .unwrap_or((0, HashMap::new()))
            .1;

        let mut all_team_members = vec![];

        for (user_id, payouts_split) in org_team_members {
            if !team_members.contains_key(&user_id) {
                all_team_members.push((user_id, payouts_split));
            }
        }
        for (user_id, payouts_split) in team_members {
            all_team_members.push((user_id, payouts_split));
        }

        // if all team members are set to zero, we treat as an equal revenue distribution
        if all_team_members.iter().all(|x| x.1 == Decimal::ZERO) {
            all_team_members
                .iter_mut()
                .for_each(|x| x.1 = Decimal::from(1));
        }

        projects_map.insert(
            project_id,
            Project {
                team_members: all_team_members,
            },
        );
    }

    let aditude_res = make_aditude_request(
        &["METRIC_IMPRESSIONS", "METRIC_REVENUE"],
        "Yesterday",
        "1d",
    )
    .await?;

    let aditude_amount: Decimal = aditude_res
        .iter()
        .map(|x| {
            x.points_list
                .iter()
                .filter_map(|x| x.metric.revenue)
                .sum::<Decimal>()
        })
        .sum();
    let aditude_impressions: u128 = aditude_res
        .iter()
        .map(|x| {
            x.points_list
                .iter()
                .filter_map(|x| x.metric.impressions)
                .sum::<u128>()
        })
        .sum();

    // Modrinth's share of ad revenue
    let modrinth_cut = Decimal::from(1) / Decimal::from(4);
    // Clean.io fee (ad antimalware). Per 1000 impressions. 0.008 CPM
    let clean_io_fee = Decimal::from(8) / Decimal::from(1000);
    // Google Ad Manager fee. Per 1000 impressions. 0.015400 CPM
    let gam_fee = Decimal::from(154) / Decimal::from(10000);

    let net_revenue = aditude_amount
        - ((clean_io_fee + gam_fee) * Decimal::from(aditude_impressions)
            / Decimal::from(1000));

    let payout = net_revenue * (Decimal::from(1) - modrinth_cut);

    // Ad payouts are Net 60 from the end of the month
    let available = {
        let now = Utc::now().date_naive();

        let year = now.year();
        let month = now.month();

        // Get the first day of the next month
        let last_day_of_month = if month == 12 {
            Utc.with_ymd_and_hms(year + 1, 1, 1, 0, 0, 0).unwrap()
        } else {
            Utc.with_ymd_and_hms(year, month + 1, 1, 0, 0, 0).unwrap()
        };

        last_day_of_month + Duration::days(59)
    };

    let (
        mut insert_user_ids,
        mut insert_project_ids,
        mut insert_payouts,
        mut insert_starts,
        mut insert_availables,
    ) = (Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new());
    for (id, project) in projects_map {
        if let Some(value) = &multipliers.values.get(&(id as u64)) {
            let project_multiplier: Decimal =
                Decimal::from(**value) / Decimal::from(multipliers.sum);

            let sum_splits: Decimal =
                project.team_members.iter().map(|x| x.1).sum();

            if sum_splits > Decimal::ZERO {
                for (user_id, split) in project.team_members {
                    let payout: Decimal =
                        payout * project_multiplier * (split / sum_splits);

                    if payout > Decimal::ZERO {
                        insert_user_ids.push(user_id);
                        insert_project_ids.push(id);
                        insert_payouts.push(payout);
                        insert_starts.push(start);
                        insert_availables.push(available);
                    }
                }
            }
        }
    }

    sqlx::query!(
        "
        INSERT INTO payouts_values (user_id, mod_id, amount, created, date_available)
        SELECT * FROM UNNEST ($1::bigint[], $2::bigint[], $3::numeric[], $4::timestamptz[], $5::timestamptz[])
        ",
        &insert_user_ids[..],
        &insert_project_ids[..],
        &insert_payouts[..],
        &insert_starts[..],
        &insert_availables[..]
    )
    .execute(&mut *transaction)
    .await?;

    transaction.commit().await?;

    Ok(())
}

// Used for testing, should be the same as the above function
pub async fn insert_payouts(
    insert_user_ids: Vec<i64>,
    insert_project_ids: Vec<i64>,
    insert_payouts: Vec<Decimal>,
    insert_starts: Vec<DateTime<Utc>>,
    insert_availables: Vec<DateTime<Utc>>,
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
) -> sqlx::Result<PgQueryResult> {
    sqlx::query!(
        "
        INSERT INTO payouts_values (user_id, mod_id, amount, created, date_available)
        SELECT * FROM UNNEST ($1::bigint[], $2::bigint[], $3::numeric[], $4::timestamptz[], $5::timestamptz[])
        ",
        &insert_user_ids[..],
        &insert_project_ids[..],
        &insert_payouts[..],
        &insert_starts[..],
        &insert_availables[..],
    )
    .execute(&mut **transaction)
    .await
}

pub async fn insert_bank_balances(
    payouts: &PayoutsQueue,
    pool: &PgPool,
) -> Result<(), ApiError> {
    let mut transaction = pool.begin().await?;

    let (paypal, brex, tremendous) = futures::future::try_join3(
        PayoutsQueue::get_paypal_balance(),
        PayoutsQueue::get_brex_balance(),
        payouts.get_tremendous_balance(),
    )
    .await?;

    let mut insert_account_types = Vec::new();
    let mut insert_amounts = Vec::new();
    let mut insert_pending = Vec::new();
    let mut insert_recorded = Vec::new();

    let now = Utc::now();
    let today = now.date_naive().and_time(NaiveTime::MIN).and_utc();

    let mut add_balance =
        |account_type: &str, balance: Option<AccountBalance>| {
            if let Some(balance) = balance {
                insert_account_types.push(account_type.to_string());
                insert_amounts.push(balance.available);
                insert_pending.push(false);
                insert_recorded.push(today);

                insert_account_types.push(account_type.to_string());
                insert_amounts.push(balance.pending);
                insert_pending.push(true);
                insert_recorded.push(today);
            }
        };

    add_balance("paypal", paypal);
    add_balance("brex", brex);
    add_balance("tremendous", tremendous);

    sqlx::query!(
        "
        INSERT INTO payout_sources_balance (account_type, amount, pending, recorded)
        SELECT * FROM UNNEST ($1::text[], $2::numeric[], $3::boolean[], $4::timestamptz[])
        ON CONFLICT (recorded, account_type, pending)
        DO UPDATE SET amount = EXCLUDED.amount
        ",
        &insert_account_types[..],
        &insert_amounts[..],
        &insert_pending[..],
        &insert_recorded[..],
    )
        .execute(&mut *transaction)
        .await?;

    transaction.commit().await?;

    Ok(())
}
