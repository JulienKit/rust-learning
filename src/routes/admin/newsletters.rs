use std::ops::DerefMut;
use crate::authentication::UserId;
use crate::routes::admin::dashboard::get_username;
use crate::utils::{e400, e500, see_other};
use actix_web::http::header::ContentType;
use actix_web::web::ReqData;
use actix_web::{HttpResponse, get, post, web};
use actix_web_flash_messages::{FlashMessage, IncomingFlashMessages};
use anyhow::Context;
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;
use crate::idempotency::{save_response, try_processing, IdempotencyKey, NextAction};

fn success_message() -> FlashMessage {
    FlashMessage::info("The newsletter issue has been published!")
}

#[get("/newsletters")]
pub async fn publish_newsletter_form(
    flash_messages: IncomingFlashMessages,
) -> Result<HttpResponse, actix_web::Error> {
    let mut msg_html = String::new();
    for m in flash_messages.iter() {
        msg_html.push_str(format!(r#"<p><i>{}</i></p>"#, m.content()).as_str());
    }
    let idempotency_key = uuid::Uuid::new_v4();
    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta http-equiv="content-type" content="text/html; charset=utf-8">
<title>Admin dashboard</title>
</head>
<body>
<h1>Send a newsletter issue</h1>
{msg_html}
<form method="POST" action="/admin/newsletters">
<input type="text" name="title">
<textarea name="html_content"></textarea>
<textarea name="text_content"></textarea>
<input hidden type="text" name="idempotency_key" value="{idempotency_key}">
<button type="submit">Publish</button>
</form>
</body>
</html>"#
        )))
}

#[derive(serde::Deserialize)]
pub struct FormData {
    title: String,
    html_content: String,
    text_content: String,
    idempotency_key: String
}

#[tracing::instrument(
    name = "Publish a newsletter issue",
    skip(form, pool),
    fields(username=tracing::field::Empty, user_id=tracing::field::Empty)
)]
#[post("/newsletters")]
pub async fn publish_newsletter(
    form: web::Form<FormData>,
    pool: web::Data<PgPool>,
    user_id: ReqData<UserId>,
) -> Result<HttpResponse, actix_web::Error> {
    let user_id = user_id.into_inner();
    let username = get_username(*user_id, &pool).await.map_err(e500)?;

    tracing::Span::current().record("username", tracing::field::display(&username));
    tracing::Span::current().record("user_id", tracing::field::display(&user_id));

    let idempotency_key: IdempotencyKey = form.0.idempotency_key.try_into().map_err(e400)?;
    let mut transaction = match try_processing(&pool, &idempotency_key, *user_id)
        .await
        .map_err(e500)?
    {
        NextAction::StartProcessing(t) => t,
        NextAction::ReturnSavedResponse(saved_response) => {
            success_message().send();
            return Ok(saved_response);
        }
    };
    let issue_id = insert_newsletter_issue(&mut transaction, &form.0.title, &form.0.text_content, &form.0.html_content)
        .await
        .context("Failed to insert newsletter issue")
        .map_err(e500)?;

    enqueue_delivery_tasks(&mut transaction, issue_id)
        .await
        .context("Failed to enqueue delivery tasks")
        .map_err(e500)?;

    let response = see_other("/admin/newsletters");

    let response = save_response(transaction, &idempotency_key, *user_id, response)
        .await
        .map_err(e500)?;
    success_message().send();
    Ok(response)
}


#[tracing::instrument(skip_all)]
async fn insert_newsletter_issue(
    transaction: &mut Transaction<'_, Postgres>,
    title: &str,
    text_content: &str,
    html_content: &str,
) -> Result<Uuid, sqlx::Error> {
    let newsletter_issue_id = Uuid::new_v4();
    sqlx::query!(
        r#"
        INSERT INTO newsletter_issues (newsletter_issue_id, title, text_content, html_content, published_at)
        VALUES ($1, $2, $3, $4, now())
        "#,
        newsletter_issue_id,
        title,
        text_content,
        html_content
    )
        .execute(transaction.deref_mut())
        .await?;
    Ok(newsletter_issue_id)
}

#[tracing::instrument(skip_all)]
async fn enqueue_delivery_tasks(
    transaction: &mut Transaction<'_, Postgres>,
    newsletter_issue_id: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO issue_delivery_queue (newsletter_issue_id, subscriber_email)
        SELECT $1, email
        FROM subscriptions
        WHERE status = 'confirmed'
        "#,
        newsletter_issue_id
    )
    .execute(transaction.deref_mut())
    .await?;
    Ok(())
}