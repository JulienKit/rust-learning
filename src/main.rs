
mod routes;
mod startup;
use std::net::TcpListener;
use crate::configuration::get_configuration;

mod configuration;

#[actix_web::main]
async fn main() -> std::io::Result<()> {

    let configuration = get_configuration().expect("Failed to read configuration.");

    let address = format!("127.0.0.1:{}", configuration.application_port);
    let listener = TcpListener::bind(address)
        .expect("Failed to bind 8080 port.");
    startup::run(listener)?.await
}