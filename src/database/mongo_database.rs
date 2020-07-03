use log::info;
use mongodb::error::Error;
use mongodb::options::ClientOptions;
use mongodb::Client;

pub async fn connect() -> Result<Client, Error> {
    info!("Initializing database connection");

    let mongodb_addr = dotenv::var("MONGODB_ADDR").expect("`MONGO_ADDR` not in .env");
    let mut client_options = ClientOptions::parse(&mongodb_addr).await?;
    client_options.app_name = Some("labrinth".to_string());

    Client::with_options(client_options)
}
