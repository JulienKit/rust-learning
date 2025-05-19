use crate::routes;
use actix_web::dev::Server;
use actix_web::middleware::Logger;
use actix_web::{web, App, HttpServer};
use std::net::TcpListener;
use std::sync::Once;
use sqlx::{PgPool};

static LOGGER_ONCE: Once = Once::new();

pub fn run(listener: TcpListener, connection_pool: PgPool) -> Result<Server, std::io::Error> {
    if std::env::var_os("RUST_LOG").is_none() {
        unsafe {
            std::env::set_var("RUST_LOG", "actix_web=info");
        }
    }

    dotenv::dotenv().ok();
    LOGGER_ONCE.call_once(|| {
        env_logger::init();
    });
    
    let connection_pool = web::Data::new(connection_pool);

    let server = HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .service(routes::healthcheck)
            .service(routes::subscribe)
            .app_data(connection_pool.clone())
    })
    .listen(listener)?
    .run();

    Ok(server)
}
