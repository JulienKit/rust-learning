use actix_web::http::header::ContentType;
use actix_web::{HttpResponse, get};

#[get("/")]
pub async fn index() -> HttpResponse {
    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(include_str!("home.html"))
}
