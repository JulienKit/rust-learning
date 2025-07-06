use crate::authentication::{AuthError, Credentials, UserId, validate_credentials};
use crate::routes::admin::dashboard::get_username;
use crate::utils::{e500, see_other};
use actix_web::http::header::ContentType;
use actix_web::{HttpResponse, get, post, web};
use actix_web_flash_messages::FlashMessage;
use actix_web_flash_messages::IncomingFlashMessages;
use secrecy::{ExposeSecret, Secret};
use sqlx::PgPool;

pub fn password(cfg: &mut web::ServiceConfig) {
    cfg.service(change_password_form).service(change_password);
}
#[get("/password")]
pub async fn change_password_form(
    flash_messages: IncomingFlashMessages,
) -> Result<HttpResponse, actix_web::Error> {
    let mut msg_html = String::new();
    for m in flash_messages.iter() {
        msg_html.push_str(format!(r#"<p><i>{}</i></p>"#, m.content()).as_str());
    }
    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta http-equiv="content-type" content="text/html; charset=utf-8">
<title>Change Password</title>
</head>
<body>
{msg_html}
<form action="/admin/password" method="post">
<label>Current password
<input
type="password"
placeholder="Enter current password"
name="current_password"
>
</label>
<br>
<label>New password
<input
type="password"
placeholder="Enter new password"
name="new_password"
>
</label>
<br>
<label>Confirm new password
<input
type="password"
placeholder="Type the new password again"
name="new_password_check"
>
</label>
<br>
<button type="submit">Change password</button>
</form>
<p><a href="/admin/dashboard">&lt;- Back</a></p>
</body>
</html>"#,
        )))
}

#[derive(serde::Deserialize)]
struct FormData {
    current_password: Secret<String>,
    new_password: Secret<String>,
    new_password_check: Secret<String>,
}

#[post("/password")]
pub async fn change_password(
    form: web::Form<FormData>,
    pool: web::Data<PgPool>,
    user_id: web::ReqData<UserId>,
) -> Result<HttpResponse, actix_web::Error> {
    let user_id = user_id.into_inner();

    if form.new_password.expose_secret() != form.new_password_check.expose_secret() {
        FlashMessage::error(
            "You entered two different new passwords - the field values must match.",
        )
        .send();
        return Ok(see_other("/admin/password"));
    }

    let username = get_username(*user_id, &pool).await.map_err(e500)?;

    let credentials = Credentials {
        username,
        password: form.0.current_password,
    };
    if let Err(e) = validate_credentials(credentials, &pool).await {
        return match e {
            AuthError::InvalidCredentials(_) => {
                FlashMessage::error("The current password is incorrect.").send();
                Ok(see_other("/admin/password"))
            }
            AuthError::UnexpectedError(_) => Err(e500(e)),
        };
    }

    crate::authentication::change_password(*user_id, form.0.new_password, &pool)
        .await
        .map_err(e500)?;
    FlashMessage::error("Your password has been changed.").send();

    Ok(see_other("/admin/password"))
}
