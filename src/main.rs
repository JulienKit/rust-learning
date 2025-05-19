mod routes;
mod startup;
use crate::configuration::get_configuration;
use std::net::TcpListener;
use sqlx::{PgPool};

mod configuration;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let configuration = get_configuration().expect("Failed to read configuration.");
    let db_pool = PgPool::connect(&configuration.database.connection_string())
        .await
        .expect("Failed to connect to Postgres database");
    let address = format!("127.0.0.1:{}", configuration.application_port);
    let listener = TcpListener::bind(address).expect("Failed to bind 8080 port.");
    startup::run(listener, db_pool)?.await
}
