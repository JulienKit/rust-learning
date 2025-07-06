mod dashboard;
mod logout;
mod newsletters;
mod password;

use actix_web::web;
pub use dashboard::admin_dashboard;
pub use logout::*;
pub use newsletters::newsletters;
pub use password::password;

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(admin_dashboard)
        .service(log_out)
        .configure(newsletters)
        .configure(password);
}
