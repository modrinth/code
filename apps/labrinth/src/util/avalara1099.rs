use crate::database::models::{DBUserId, users_compliance::FormType};
use crate::routes::ApiError;
use ariadne::ids::base62_impl::to_base62;
use chrono::Datelike;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Serialize, Deserialize)]
pub struct DataWrapper<T> {
    pub data: Data<T>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListWrapper<T> {
    pub data: Vec<Data<T>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Data<T> {
    #[serde(rename = "type")]
    pub r#type: Option<String>,
    pub id: Option<String>,
    pub attributes: T,
    pub links: Option<HashMap<String, String>>,
}

#[derive(Serialize, Deserialize)]
pub struct FormResponse {
    pub form_type: FormType,
    pub form_id: Option<String>,
    pub company_id: u32,
    pub company_name: String,
    pub company_email: String,
    pub reference_id: String,
    /// This is a DateTime, but it's not consistent whether it has a
    /// timezone or not, so we just parse it as a string and use [`Utc::now()`](fn@chrono::Utc::now)
    /// rather than using the provided DateTime.
    pub signed_at: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct W9FormsResponse {
    pub e_delivery_consented_at: Option<String>,
    pub tin_match_status: Option<String>,
    pub entry_status: String,
}

pub async fn request_form(
    user_id: DBUserId,
    form_type: FormType,
) -> Result<Result<DataWrapper<FormResponse>, serde_json::Value>, ApiError> {
    const DEFAULT_TTL: u32 = 3600;

    #[derive(Serialize, Deserialize)]
    struct FormRequest {
        form_type: FormType,
        company_id: String,
        reference_id: String,
        ttl: u32,
    }

    let (request_builder, company_id) =
        team_request(reqwest::Method::POST, "/form_requests")?;

    let response = request_builder
        .json(&DataWrapper {
            data: Data {
                r#type: Some("form_request".to_owned()),
                id: None,
                attributes: FormRequest {
                    form_type,
                    company_id,
                    ttl: DEFAULT_TTL,
                    reference_id: Reference {
                        user_id,
                        form_type,
                        current_year: chrono::Utc::now().year_ce().1,
                    }
                    .to_string(),
                },
                links: None,
            },
        })
        .send()
        .await?;

    Ok(if response.status().is_success() {
        Ok(response.json::<DataWrapper<FormResponse>>().await?)
    } else {
        Err(response.json().await?)
    })
}

pub async fn check_form(
    reference_id: &str,
) -> Result<
    Result<Option<DataWrapper<W9FormsResponse>>, serde_json::Value>,
    ApiError,
> {
    let (request_builder, _company_id) = team_request(
        reqwest::Method::GET,
        &format!(
            "/w9forms?filter[reference_id_eq]={reference_id}&page[number]=1&page[size]=1"
        ),
    )?;

    let response = request_builder.send().await?;

    Ok(if response.status().is_success() {
        let body = response.text().await?;
        let serde_result =
            serde_json::from_str::<ListWrapper<W9FormsResponse>>(&body);

        match serde_result {
            Ok(mut list_wrapper) => {
                Ok(list_wrapper.data.pop().map(|data| DataWrapper { data }))
            }
            Err(e) => {
                return Err(ApiError::InvalidInput(format!(
                    "Error parsing avalara1099 response: {e}. Actual response body: {body}"
                )));
            }
        }
    } else {
        Err(response.json().await?)
    })
}

fn team_request(
    method: reqwest::Method,
    route: &str,
) -> Result<(reqwest::RequestBuilder, String), ApiError> {
    let key = dotenvy::var("AVALARA_1099_API_KEY")?;
    let url = dotenvy::var("AVALARA_1099_API_URL")?;
    let team = dotenvy::var("AVALARA_1099_API_TEAM_ID")?;
    let company = dotenvy::var("AVALARA_1099_COMPANY_ID")?;

    let url = url.trim_end_matches('/');

    let client = reqwest::Client::new();

    Ok((
        client
            .request(method, format!("{url}/v1/{team}{route}"))
            .header(reqwest::header::USER_AGENT, "Modrinth")
            .bearer_auth(&key),
        company,
    ))
}

struct Reference {
    user_id: DBUserId,
    form_type: FormType,
    current_year: u32,
}

impl fmt::Display for Reference {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}_{}_{}",
            to_base62(self.user_id.0 as u64),
            self.form_type,
            self.current_year
        )
    }
}
