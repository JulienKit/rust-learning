use actix_web::{HttpResponse, Responder, post, web};

#[derive(serde::Deserialize)]
struct FormData {
    _email: String,
    _name: String,
}

#[post("/subscribe")]
async fn subscribe(_form: web::Form<FormData>) -> impl Responder {
    HttpResponse::Ok().finish()
}
