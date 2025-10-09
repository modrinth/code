use std::{env, fmt::Debug, io};

use eyre::WrapErr;
use muralpay::{
    CreatePayout, CreatePayoutDetails, Dob, FiatAccountType,
    FiatAndRailDetails, MuralPay, PayoutPurpose, PayoutRecipientInfo,
    PhysicalAddress, SupportingDetails, TokenAmount, UsdSymbol,
};
use rust_decimal::dec;
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
        source_account_id: String,
        /// Description for this payout request
        memo: Option<String>,
    },
}

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
enum OutputFormat {
    Json,
    JsonMin,
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
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
            muralpay
                .create_payout_request(
                    source_account_id
                        .parse()
                        .wrap_err("invalid source account ID")?,
                    memo,
                    &[CreatePayout {
                        amount: TokenAmount {
                            token_amount: dec!(2.00),
                            token_symbol: "USDC".into(),
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
                                country: "US".into(),
                                state: "CA".into(),
                                city: "Springfield".into(),
                                zip: "90001".into(),
                            },
                        },
                        supporting_details: Some(SupportingDetails {
                            supporting_document: Some("data:image/jpeg;base64,/9j/4AAQSkZJRgABAQAAAQABAA...".into()),
                            payout_purpose: Some(PayoutPurpose::Payroll),
                        }),
                    }],
                )
                .await?,
        ),
    }

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
