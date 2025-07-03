use learning::configuration::get_configuration;
use learning::startup::Application;
use learning::telemetry::{init_telemetry};

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    
    let app_name = std::env::var("CARGO_PKG_NAME").unwrap_or("learning api".into());

    init_telemetry(&app_name, &"learning-api-tracer".to_string())?;
    
    let configuration = get_configuration().expect("Failed to read configuration.");

    let application = Application::build(configuration).await?;
    application.run_until_stopped().await?;
    Ok(())
}
