use log::info;
use mongodb::error::Error;
use mongodb::options::ClientOptions;
use mongodb::Client;

pub async fn connect() -> Result<Client, Error> {
    info!("Initializing database connection");

    let mut client_options = ClientOptions::parse(&dotenv::var("MONGODB_ADDR").unwrap()).await?;
    client_options.app_name = Some("labrinth".to_string());

    Client::with_options(client_options)
}
