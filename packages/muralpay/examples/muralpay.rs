use std::{env, fmt::Debug, io};

use eyre::{Result, WrapErr, eyre};
use muralpay::{
    AccountId, CounterpartyId, CreatePayout, CreatePayoutDetails, Dob,
    FiatAccountType, FiatAndRailCode, FiatAndRailDetails, FiatFeeRequest,
    FiatPayoutFee, MuralPay, PayoutMethodId, PayoutRecipientInfo,
    PhysicalAddress, TokenAmount, TokenFeeRequest, TokenPayoutFee, UsdSymbol,
};
use rust_decimal::{Decimal, dec};
use serde::Serialize;

#[derive(Debug, clap::Parser)]
struct Args {
    #[arg(short, long)]
    output: Option<OutputFormat>,
    #[clap(subcommand)]
    command: Command,
}

#[derive(Debug, clap::Subcommand)]
enum Command {
    /// Account listing and management
    Account {
        #[command(subcommand)]
        command: AccountCommand,
    },
    /// Payouts and payout requests
    Payout {
        #[command(subcommand)]
        command: PayoutCommand,
    },
    /// Counterparty management
    Counterparty {
        #[command(subcommand)]
        command: CounterpartyCommand,
    },
    /// Payout method management
    PayoutMethod {
        #[command(subcommand)]
        command: PayoutMethodCommand,
    },
}

#[derive(Debug, clap::Subcommand)]
enum AccountCommand {
    /// List all accounts
    #[clap(alias = "ls")]
    List,
}

#[derive(Debug, clap::Subcommand)]
enum PayoutCommand {
    /// List all payout requests
    #[clap(alias = "ls")]
    List,
    /// Create a payout request
    Create {
        /// ID of the Mural account to send from
        source_account_id: AccountId,
        /// Description for this payout request
        memo: Option<String>,
    },
    /// Get fees for a transaction
    Fees {
        #[command(subcommand)]
        command: PayoutFeesCommand,
    },
    /// Get bank details for a fiat and rail code
    BankDetails {
        /// Fiat and rail code to fetch bank details for
        fiat_and_rail_code: FiatAndRailCode,
    },
}

#[derive(Debug, clap::Subcommand)]
enum PayoutFeesCommand {
    /// Get fees for a token-to-fiat transaction
    Token {
        amount: Decimal,
        fiat_and_rail_code: FiatAndRailCode,
    },
    /// Get fees for a fiat-to-token transaction
    Fiat {
        amount: Decimal,
        fiat_and_rail_code: FiatAndRailCode,
    },
}

#[derive(Debug, clap::Subcommand)]
enum CounterpartyCommand {
    /// List all counterparties
    #[clap(alias = "ls")]
    List,
}

#[derive(Debug, clap::Subcommand)]
enum PayoutMethodCommand {
    /// List payout methods for a counterparty
    #[clap(alias = "ls")]
    List {
        /// ID of the counterparty
        counterparty_id: CounterpartyId,
    },
    /// Delete a payout method
    Delete {
        /// ID of the counterparty
        counterparty_id: CounterpartyId,
        /// ID of the payout method to delete
        payout_method_id: PayoutMethodId,
    },
}

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
enum OutputFormat {
    Json,
    JsonMin,
}

#[tokio::main]
async fn main() -> Result<()> {
    _ = dotenvy::dotenv();
    color_eyre::install().expect("failed to install `color-eyre`");
    tracing_subscriber::fmt().init();

    let args = <Args as clap::Parser>::parse();
    let of = args.output;

    let api_url = env::var("MURALPAY_API_URL")
        .unwrap_or_else(|_| muralpay::SANDBOX_API_URL.to_string());
    let api_key = env::var("MURALPAY_API_KEY").wrap_err("no API key")?;
    let transfer_api_key = env::var("MURALPAY_TRANSFER_API_KEY").ok();

    let muralpay = MuralPay::new(api_url, api_key, transfer_api_key);

    match args.command {
        Command::Account {
            command: AccountCommand::List,
        } => run(of, muralpay.get_all_accounts().await?),
        Command::Payout {
            command: PayoutCommand::List,
        } => run(of, muralpay.search_payout_requests(None, None).await?),
        Command::Payout {
            command:
                PayoutCommand::Create {
                    source_account_id,
                    memo,
                },
        } => run(
            of,
            create_payout_request(
                &muralpay,
                source_account_id,
                memo.as_deref(),
            )
            .await?,
        ),
        Command::Payout {
            command:
                PayoutCommand::Fees {
                    command:
                        PayoutFeesCommand::Token {
                            amount,
                            fiat_and_rail_code,
                        },
                },
        } => run(
            of,
            get_fees_for_token_amount(&muralpay, amount, fiat_and_rail_code)
                .await?,
        ),
        Command::Payout {
            command:
                PayoutCommand::Fees {
                    command:
                        PayoutFeesCommand::Fiat {
                            amount,
                            fiat_and_rail_code,
                        },
                },
        } => run(
            of,
            get_fees_for_fiat_amount(&muralpay, amount, fiat_and_rail_code)
                .await?,
        ),
        Command::Payout {
            command: PayoutCommand::BankDetails { fiat_and_rail_code },
        } => run(of, muralpay.get_bank_details(&[fiat_and_rail_code]).await?),
        Command::Counterparty {
            command: CounterpartyCommand::List,
        } => run(of, list_counterparties(&muralpay).await?),
        Command::PayoutMethod {
            command: PayoutMethodCommand::List { counterparty_id },
        } => run(
            of,
            muralpay
                .search_payout_methods(counterparty_id, None)
                .await?,
        ),
        Command::PayoutMethod {
            command:
                PayoutMethodCommand::Delete {
                    counterparty_id,
                    payout_method_id,
                },
        } => run(
            of,
            muralpay
                .delete_payout_method(counterparty_id, payout_method_id)
                .await?,
        ),
    }

    Ok(())
}

async fn create_payout_request(
    muralpay: &MuralPay,
    source_account_id: AccountId,
    memo: Option<&str>,
) -> Result<()> {
    muralpay
        .create_payout_request(
            source_account_id,
            memo,
            &[CreatePayout {
                amount: TokenAmount {
                    token_amount: dec!(2.00),
                    token_symbol: muralpay::USDC.into(),
                },
                payout_details: CreatePayoutDetails::Fiat {
                    bank_name: "Foo Bank".into(),
                    bank_account_owner: "John Smith".into(),
                    developer_fee: None,
                    fiat_and_rail_details: FiatAndRailDetails::Usd {
                        symbol: UsdSymbol::Usd,
                        account_type: FiatAccountType::Checking,
                        bank_account_number: "123456789".into(),
                        // idk what the format is, https://wise.com/us/routing-number/bank/us-bank
                        bank_routing_number: "071004200".into(),
                    },
                },
                recipient_info: PayoutRecipientInfo::Individual {
                    first_name: "John".into(),
                    last_name: "Smith".into(),
                    email: "john.smith@example.com".into(),
                    date_of_birth: Dob::new(1970, 1, 1).unwrap(),
                    physical_address: PhysicalAddress {
                        address1: "1234 Elm Street".into(),
                        address2: Some("Apt 56B".into()),
                        country: rust_iso3166::US,
                        state: "CA".into(),
                        city: "Springfield".into(),
                        zip: "90001".into(),
                    },
                },
                supporting_details: None,
            }],
        )
        .await?;
    Ok(())
}

async fn get_fees_for_token_amount(
    muralpay: &MuralPay,
    amount: Decimal,
    fiat_and_rail_code: FiatAndRailCode,
) -> Result<TokenPayoutFee> {
    let fees = muralpay
        .get_fees_for_token_amount(&[TokenFeeRequest {
            amount: TokenAmount {
                token_amount: amount,
                token_symbol: muralpay::USDC.into(),
            },
            fiat_and_rail_code,
        }])
        .await?;
    let fee = fees
        .into_iter()
        .next()
        .ok_or_else(|| eyre!("no fee results returned"))?;
    Ok(fee)
}

async fn get_fees_for_fiat_amount(
    muralpay: &MuralPay,
    amount: Decimal,
    fiat_and_rail_code: FiatAndRailCode,
) -> Result<FiatPayoutFee> {
    let fees = muralpay
        .get_fees_for_fiat_amount(&[FiatFeeRequest {
            fiat_amount: amount,
            token_symbol: muralpay::USDC.into(),
            fiat_and_rail_code,
        }])
        .await?;
    let fee = fees
        .into_iter()
        .next()
        .ok_or_else(|| eyre!("no fee results returned"))?;
    Ok(fee)
}

async fn list_counterparties(muralpay: &MuralPay) -> Result<()> {
    let _counterparties = muralpay.search_counterparties(None).await?;
    Ok(())
}

fn run<T: Debug + Serialize>(output_format: Option<OutputFormat>, value: T) {
    match output_format {
        None => {
            println!("{value:#?}");
        }
        Some(OutputFormat::Json) => {
            _ = serde_json::to_writer_pretty(io::stdout(), &value)
        }
        Some(OutputFormat::JsonMin) => {
            _ = serde_json::to_writer(io::stdout(), &value);
        }
    }
}
