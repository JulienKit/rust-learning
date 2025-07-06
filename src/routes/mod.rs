mod admin;
mod health_check;
mod home;
mod login;
mod subscriptions;
mod subscriptions_confirm;

use crate::authentication::reject_anonymous_users;
use actix_web::middleware::from_fn;
use actix_web::web;
use home::*;

pub fn get(cfg: &mut web::ServiceConfig) {
    cfg.service(health_check::healthcheck)
        .service(subscriptions::subscribe)
        .service(subscriptions_confirm::confirm)
        .service(index)
        .configure(login::routes)
        .service(
            web::scope("/admin")
                .wrap(from_fn(reject_anonymous_users))
                .configure(admin::routes),
        );
}
