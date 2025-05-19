use actix_web::{HttpResponse, Responder, post, web};
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(serde::Deserialize)]
struct FormData {
    email: String,
    name: String,
}

#[post("/subscribe")]
async fn subscribe(form: web::Form<FormData>, db_pool:web::Data<PgPool>) -> impl Responder {

    match sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at) VALUES ($1, $2, $3, $4);"#,
        Uuid::new_v4(),
        form.email,
        form.name,
        Utc::now()
    )
        .execute(db_pool.get_ref())
        .await
    {
        Ok(_) =>   HttpResponse::Ok().finish(),
        Err(e) => {
            println!("Failed to execute query {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }


}
