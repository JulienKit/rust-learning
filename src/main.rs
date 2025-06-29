use learning::configuration::get_configuration;
use learning::startup::Application;
use learning::telemetry::{get_subscriber, init_subscriber};

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    let subscriber = get_subscriber("learning".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);
    let configuration = get_configuration().expect("Failed to read configuration.");

    let application = Application::build(configuration).await?;
    application.run_until_stopped().await?;
    Ok(())
}
