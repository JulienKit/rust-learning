#[cfg(test)]
mod tests {
    use learning::configuration::get_configuration;
    use learning::startup;
    use reqwest::Client;
    use sqlx::{PgPool};
    use std::net::TcpListener;

    struct TestApp {
        address: String,
        db_pool: PgPool
    }
    async fn spawn_app() -> TestApp {
        let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
        let configuration = get_configuration().expect("Failed to load configuration");
        let port = listener.local_addr().unwrap().port();
        let connection = PgPool::connect(&configuration.database.connection_string())
            .await
            .expect("Failed to connect to Postgres db");
        let server = startup::run(listener, connection.clone()).expect("Failed to bind address");
        let _ = actix_rt::spawn(server);
        TestApp {
            address: format!("http://127.0.0.1:{}", port),
            db_pool: connection
        }
    }
    
    #[actix_web::test]
    async fn health_check_works() {
        
        let app = spawn_app()
            .await;

        let response = Client::new()
            .get(&format!("{}/healthcheck", &app.address))
            .send()
            .await
            .expect("Failed to execute request.");

        assert!(response.status().is_success());
        assert_eq!(Some(0), response.content_length());
    }

    #[actix_web::test]
    async fn subscribe_returns_a_200_for_valid_form_data() {
       let client = Client::new();

        let app = spawn_app().await;

        let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
        let response = client
            .post(&format!("{}/subscribe", &app.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request.");

        assert_eq!(200, response.status().as_u16());

        let saved = sqlx::query!("SELECT email, name FROM subscriptions")
            .fetch_one(&app.db_pool)
            .await
            .expect("Failed to fetch saved subscription");

        assert_eq!(saved.email, "ursula_le_guin@gmail.com");
        assert_eq!(saved.name, "le guin");
    }

    #[actix_web::test]
    async fn subscribe_returns_a_400_when_data_is_missing() {
        let client = Client::new();
        let app = spawn_app().await;
        let test_cases = vec![
            ("name=le%20guin", "missing the email"),
            ("email=ursula_le_guin%40gmail.com", "missing the name"),
            ("", "missing both name and email"),
        ];

        for (invalid_body, error_message) in test_cases {
            let response = client
                .post(&format!("{}/subscribe", &app.address))
                .header("Content-Type", "application/x-www-form-urlencoded")
                .body(invalid_body)
                .send()
                .await
                .expect("Failed to execute request.");

            assert_eq!(
                400,
                response.status().as_u16(),
                "The API did not fail with 400 Bad Request when the payload was {}.",
                error_message
            )
        }
    }
}
