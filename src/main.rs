use zero2prod::startup::{run, pre_run_build};
use zero2prod::configurations::get_configurations;
use zero2prod::telemetry::{get_tracing_subscriber, init_tracing_subscriber};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
	let subscriber = get_tracing_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
	init_tracing_subscriber(subscriber);

	let configs = get_configurations().expect("Unable to load configs");
	let (listener, email_client, db_pool) = pre_run_build(configs).await?;

    run(listener, db_pool, email_client)?.await
}
