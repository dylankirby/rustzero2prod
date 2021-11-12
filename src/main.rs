use zero2prod::startup::Application;
use zero2prod::configurations::get_configurations;
use zero2prod::telemetry::{get_tracing_subscriber, init_tracing_subscriber};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
	let subscriber = get_tracing_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
	init_tracing_subscriber(subscriber);

	let configs = get_configurations().expect("Unable to load configs");
	let application = Application::build(configs).await?;
	application.run_server().await?;

	Ok(())
}
