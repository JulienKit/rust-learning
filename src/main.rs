use learning::configuration::get_configuration;
use learning::startup;
use learning::telemetry::{get_subscriber, init_subscriber};
use sqlx::postgres::PgPoolOptions;
use std::net::TcpListener;
use learning::email_client::EmailClient;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let subscriber = get_subscriber("learning".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);
    let configuration = get_configuration().expect("Failed to read configuration.");

    println!("configuration = {:?}", &configuration);
    let db_pool = PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_secs(2))
        .connect_lazy(&configuration.database.connection_string())
        .expect("Failed to connect to Postgres database");
    let address = format!(
        "{}:{}",
        configuration.application.host, configuration.application.port
    );
    let prepared_expect_msg = format!("Failed to bind {} to listener", &address);

    let listener = TcpListener::bind(&address).expect(&prepared_expect_msg);
    let timeout = configuration.email_client.timeout();
    let sender_email = configuration.email_client.sender().expect("Invalid sender email address.");
    let email_client = EmailClient::new(
        configuration.email_client.base_url,
        sender_email,
        configuration.email_client.authorization_token,
        timeout
    );

    startup::run(listener, db_pool, email_client)?.await
}
