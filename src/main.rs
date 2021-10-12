use std::net::TcpListener;

use sqlx::postgres::PgPoolOptions;

use zero2prod::startup::run;
use zero2prod::configurations::get_configurations;
use zero2prod::telemetry::{get_tracing_subscriber, init_tracing_subscriber};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
	let subscriber = get_tracing_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
	init_tracing_subscriber(subscriber);

	let configs = get_configurations().expect("Unable to load configs");
	let db_connection_url = configs.database.connection_url();
	let db_pool = PgPoolOptions::new()
		.connect_timeout(std::time::Duration::from_secs(2))
		.connect(&db_connection_url)
		.await
		.expect("Failed to connect to Postgres");

	let application_address = format!("{}:{}", configs.application.host, configs.application.port);
	let listener = TcpListener::bind(application_address)?;
    run(listener, db_pool)?.await
}
