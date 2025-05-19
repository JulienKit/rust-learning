use std::net::TcpListener;
use std::sync::Once;
use actix_web::dev::Server;
use actix_web::{App, HttpServer};
use actix_web::middleware::Logger;
use crate::routes;
use sqlx::PgConnection;

static LOGGER_ONCE: Once = Once::new();

pub fn run(listener: TcpListener) -> Result<Server, std::io::Error> {

    if std::env::var_os("RUST_LOG").is_none() {
        unsafe { std::env::set_var("RUST_LOG", "actix_web=info"); }
    }

    dotenv::dotenv().ok();
    LOGGER_ONCE.call_once(|| {
        env_logger::init();
    });

    let server = HttpServer::new(|| {
        App::new()
            .wrap(Logger::default())
            .service(routes::healthcheck)
            .service(routes::subscribe)
    })
        .listen(listener)?
        .run();

    Ok(server)
}