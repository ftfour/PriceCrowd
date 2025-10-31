use anyhow::Result;
use mongodb::{options::ClientOptions, Client, Database};

// Тонкая обертка над инициализацией MongoDB, чтобы разгрузить state.rs.
// TODO: вынести в отдельный сервис позже (отдельный crate для БД-адаптеров)

pub async fn create_client(uri: &str, app_name: &str) -> Result<Client> {
    let mut opts = ClientOptions::parse(uri).await?;
    opts.app_name = Some(app_name.to_string());
    let client = Client::with_options(opts)?;
    Ok(client)
}

pub fn get_database(client: &Client, name: &str) -> Database {
    client.database(name)
}
