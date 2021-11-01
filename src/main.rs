use std::net::TcpListener;

use sqlx::postgres::PgPoolOptions;

use zero2prod::startup::run;
use zero2prod::configurations::get_configurations;
use zero2prod::telemetry::{get_tracing_subscriber, init_tracing_subscriber};
use zero2prod::email_client::EmailClient;

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

	let sender_email = configs.email_client.get_sender_email()
		.expect("Failed to parse sender email, seems invalid");

	let email_client = EmailClient::new(
		configs.email_client.base_url,
		sender_email,
		configs.email_client.authorization_token
	);

	let application_address = format!("{}:{}", configs.application.host, configs.application.port);
	let listener = TcpListener::bind(application_address)?;
    run(listener, db_pool, email_client)?.await
}
