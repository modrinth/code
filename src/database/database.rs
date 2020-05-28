use mongodb::options::ClientOptions;
use mongodb::Client;
use mongodb::error::Error;

pub async fn connect() -> Result<Client, Error> {
    info!("Initializing database connection");

    let mut client_options = ClientOptions::parse("mongodb://localhost:27017").await?;
    client_options.app_name = Some("Actix Web Server".to_string());

   Client::with_options(client_options)
}
