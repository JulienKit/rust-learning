mod routes;
mod startup;
use crate::configuration::get_configuration;
use std::net::TcpListener;
mod configuration;
use learning::telemetry::{get_subscriber, init_subscriber};
use sqlx::postgres::PgPoolOptions;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let subscriber = get_subscriber("learning".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);
    let configuration = get_configuration().expect("Failed to read configuration.");
    let db_pool = PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_secs(2))
        .connect_lazy(&configuration.database.connection_string())
        .expect("Failed to connect to Postgres database");
    let address = format!(
        "{}:{}",
        configuration.application.host ,configuration.application.port
    );
    let listener = TcpListener::bind(address).expect("Failed to bind 8080 port.");
    startup::run(listener, db_pool)?.await
}
