use crate::routes::internal::gotenberg::{
    GotenbergDocument, ModrinthPaymentId,
};
use crate::routes::{ApiError, internal::gotenberg::GotenbergQueue};
use crate::util::env::env_var;
use crate::util::error::Context;
use actix_web::{http::header::HeaderName, web};
use chrono::{DateTime, Datelike, Utc};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::time::Duration;
use tokio::sync::oneshot;
use tokio::time::timeout;
use tracing::warn;

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
    pub payment_date: DateTime<Utc>,
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
    queue: web::Data<GotenbergQueue>,
}

impl GotenbergClient {
    /// Initialize the client from environment variables.
    pub fn from_env(queue: web::Data<GotenbergQueue>) -> eyre::Result<Self> {
        let client = reqwest::Client::builder()
            .user_agent("Modrinth")
            .build()
            .wrap_err("failed to build reqwest client")?;

        let gotenberg_url = env_var("GOTENBERG_URL")?;
        let site_url = env_var("SITE_URL")?;
        let callback_base = env_var("GOTENBERG_CALLBACK_BASE")?;

        Ok(Self {
            client,
            gotenberg_url: gotenberg_url.trim_end_matches('/').to_owned(),
            site_url: site_url.trim_end_matches('/').to_owned(),
            callback_base: callback_base.trim_end_matches('/').to_owned(),
            queue,
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

    /// Tells Gotenberg to generate a payment statement PDF, and waits until we
    /// get a response for that PDF.
    ///
    /// This submits the PDF via [`GotenbergClient::generate_payment_statement`]
    /// then waits until we receive a response on the Gotenberg webhook, and
    /// returns that from this function.
    ///
    /// If Gotenberg does not return a response to us within `GOTENBERG_TIMEOUT`
    /// number of milliseconds, this will fail.
    pub async fn wait_for_payment_statement(
        &self,
        statement: &PaymentStatement,
    ) -> Result<GotenbergDocument, ApiError> {
        let (tx_result, rx_result) = oneshot::channel();
        let payment_id = ModrinthPaymentId(statement.payment_id.clone());

        let old = self.queue.pending.insert(payment_id.clone(), tx_result);
        if old.is_some() {
            warn!(
                "Overwrote pending document generation {payment_id:?}; \
                a previous payment statement generation operation will have its sender dropped!"
            );
        }

        self.generate_payment_statement(statement).await?;

        let timeout_ms = env_var("GOTENBERG_TIMEOUT")
            .map_err(ApiError::Internal)?
            .parse::<u64>()
            .wrap_internal_err(
                "`GOTENBERG_TIMEOUT` is not a valid number of milliseconds",
            )?;

        let document = timeout(Duration::from_millis(timeout_ms), rx_result)
            .await
            .wrap_internal_err("Gotenberg generation timed out")?
            .wrap_internal_err("Gotenberg document sender dropped")?
            .wrap_internal_err("Gotenberg document generation failed")?;
        Ok(document)
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
        (
            "statement.payment_date",
            format!(
                "{:04}-{:02}-{:02}",
                s.payment_date.year(),
                s.payment_date.month(),
                s.payment_date.day()
            ),
        ),
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
