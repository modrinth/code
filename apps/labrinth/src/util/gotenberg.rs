use crate::routes::ApiError;
use crate::util::error::Context;
use actix_web::http::header::HeaderName;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

pub const MODRINTH_GENERATED_PDF_TYPE: HeaderName =
    HeaderName::from_static("modrinth-generated-pdf-type");
pub const MODRINTH_PAYMENT_ID: HeaderName =
    HeaderName::from_static("modrinth-payment-id");

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PaymentStatement {
    pub payment_id: String,
    pub recipient_address_line_1: Option<String>,
    pub recipient_address_line_2: Option<String>,
    pub recipient_address_line_3: Option<String>,
    pub recipient_email: String,
    pub payment_date: String,
    pub gross_amount_cents: i64,
    pub net_amount_cents: i64,
    pub fees_cents: i64,
    pub currency_code: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GeneratedPdfType {
    PaymentStatement,
}

impl GeneratedPdfType {
    pub fn as_str(self) -> &'static str {
        match self {
            GeneratedPdfType::PaymentStatement => "payment-statement",
        }
    }
}

impl FromStr for GeneratedPdfType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "payment-statement" => Ok(GeneratedPdfType::PaymentStatement),
            _ => Err(s.to_owned()),
        }
    }
}

#[derive(Clone)]
pub struct GotenbergClient {
    client: reqwest::Client,
    gotenberg_url: String,
    site_url: String,
    callback_base: String,
}

impl GotenbergClient {
    /// Initialize the client from environment variables.
    pub fn from_env() -> Result<Self, ApiError> {
        let client = reqwest::Client::builder()
            .user_agent("Modrinth")
            .build()
            .wrap_internal_err("failed to build reqwest client")?;

        let gotenberg_url = dotenvy::var("GOTENBERG_URL")
            .wrap_internal_err("GOTENBERG_URL is not set")?;
        let site_url = dotenvy::var("SITE_URL")
            .wrap_internal_err("SITE_URL is not set")?;
        let callback_base = dotenvy::var("GOTENBERG_CALLBACK_BASE")
            .wrap_internal_err("GOTENBERG_CALLBACK_BASE is not set")?;

        Ok(Self {
            client,
            gotenberg_url: gotenberg_url.trim_end_matches('/').to_owned(),
            site_url: site_url.trim_end_matches('/').to_owned(),
            callback_base: callback_base.trim_end_matches('/').to_owned(),
        })
    }

    /// Generate a PDF payment statement via Gotenberg.
    ///
    /// This will:
    /// - Fetch the HTML template from `{SITE_URL}/_internal/templates/doc/payment-statement`.
    /// - Perform simple template substitution with fields from `PaymentStatement`.
    /// - Submit the HTML to Gotenberg HTML route with webhook headers.
    pub async fn generate_payment_statement(
        &self,
        statement: &PaymentStatement,
    ) -> Result<(), ApiError> {
        let template_url = format!(
            "{}/_internal/templates/doc/payment-statement",
            self.site_url
        );

        let template_html = {
            let resp = self
                .client
                .get(template_url)
                .send()
                .await
                .wrap_internal_err(
                    "failed to request payment statement template",
                )?;
            let resp = resp.error_for_status().wrap_internal_err(
                "failed to fetch payment statement template (bad status)",
            )?;
            resp.text().await.wrap_internal_err(
                "failed to read payment statement template body",
            )?
        };

        let filled_html = fill_statement_template(&template_html, statement);

        let form = reqwest::multipart::Form::new().part(
            "files",
            reqwest::multipart::Part::text(filled_html)
                .file_name("index.html")
                .mime_str("text/html")
                .wrap_internal_err("invalid mime type for html part")?,
        );

        let success_webhook = format!("{}/success", self.callback_base);
        let error_webhook = format!("{}/error", self.callback_base);

        self
            .client
            .post(format!(
                "{}/forms/chromium/convert/html",
                self.gotenberg_url
            ))
            .header("Gotenberg-Webhook-Url", success_webhook)
            .header("Gotenberg-Webhook-Error-Url", error_webhook)
            .header(
                "Gotenberg-Webhook-Extra-Http-Headers",
                serde_json::json!({
					"Modrinth-Payment-Id": statement.payment_id,
					"Modrinth-Generated-Pdf-Type": GeneratedPdfType::PaymentStatement.as_str(),
				}).to_string(),
            )
            .header(
                "Modrinth-Payment-Id",
                &statement.payment_id,
            )
            .header(
                "Gotenberg-Output-Filename",
                format!("payment-statement-{}", statement.payment_id),
            )
            .multipart(form)
            .send()
            .await
            .wrap_internal_err("failed to submit HTML to Gotenberg")?
            .error_for_status()
            .wrap_internal_err("Gotenberg returned an error status")?;

        Ok(())
    }
}

fn fill_statement_template(html: &str, s: &PaymentStatement) -> String {
    let variables: Vec<(&str, String)> = vec![
        ("statement.payment_id", s.payment_id.clone()),
        (
            "statement.recipient_address_line_1",
            s.recipient_address_line_1.clone().unwrap_or_default(),
        ),
        (
            "statement.recipient_address_line_2",
            s.recipient_address_line_2.clone().unwrap_or_default(),
        ),
        (
            "statement.recipient_address_line_3",
            s.recipient_address_line_3.clone().unwrap_or_default(),
        ),
        ("statement.recipient_email", s.recipient_email.clone()),
        ("statement.payment_date", s.payment_date.clone()),
        (
            "statement.gross_amount",
            format_money(s.gross_amount_cents, &s.currency_code),
        ),
        (
            "statement.net_amount",
            format_money(s.net_amount_cents, &s.currency_code),
        ),
        (
            "statement.fees",
            format_money(s.fees_cents, &s.currency_code),
        ),
    ];

    let mut out = String::with_capacity(html.len());
    let mut remaining = html;
    while let Some((before, rest)) = remaining.split_once('{') {
        out.push_str(before);
        if let Some((key, after)) = rest.split_once('}') {
            let key = key.trim();
            if let Some((_, val)) = variables.iter().find(|(k, _)| *k == key) {
                out.push_str(val);
            }
            // if key not found, insert empty string
            remaining = after;
        } else {
            // unmatched '{', push the rest and break
            out.push_str(rest);
            remaining = "";
            break;
        }
    }
    out.push_str(remaining);
    out
}

fn format_money(amount_cents: i64, currency: &str) -> String {
    rusty_money::Money::from_minor(
        amount_cents,
        rusty_money::iso::find(currency).unwrap_or(rusty_money::iso::USD),
    )
    .to_string()
}
