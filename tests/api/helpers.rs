
use sqlx::{PgConnection, PgPool, Connection, Executor};
use uuid::Uuid;

use zero2prod::startup::{Application, build_connection_pool};
use zero2prod::configurations::{get_configurations, DatabaseSettings};
use zero2prod::telemetry::{get_tracing_subscriber, init_tracing_subscriber};

use once_cell::sync::Lazy;

static TRACING: Lazy<()> = Lazy::new(|| {
	let log_level = "debug".to_string();
	let subscriber_name = "test".to_string();

	if std::env::var("TEST_LOG").is_ok() {
		let subscriber = get_tracing_subscriber(subscriber_name, log_level, std::io::stdout);
		init_tracing_subscriber(subscriber);
	} else {
		let subscriber = get_tracing_subscriber(subscriber_name, log_level, std::io::sink);
		init_tracing_subscriber(subscriber);
	}

});

pub struct TestApp {
	pub address: String,
	pub db_pool: PgPool
}

pub async fn spawn_app() -> TestApp {
	Lazy::force(&TRACING);

	let configs = {
		let mut c = get_configurations().expect("Unable to load configs");
		c.database.database_name = Uuid::new_v4().to_string();
		c.application.port = 0;
		c
	};

	let db_pool = configure_database(&configs.database)
		.await;

	let application = Application::build(configs.clone()).await.expect("Failed to build application");

	let address = format!("http://127.0.0.1:{}", application.port());

	let _ = tokio::spawn(application.run_server());

	TestApp {
		address: address,
		db_pool: db_pool
	}
}

async fn configure_database(database_configs: &DatabaseSettings) -> PgPool{
	let mut connection = PgConnection::connect(&database_configs.connection_url_without_db())
		.await
		.expect("Failed to connect to Postgres");

	connection.execute(&*format!(r#"CREATE DATABASE "{}";"#, &database_configs.database_name))
		.await
		.expect("Failed to executre create database command on test startup");

	let db_pool = build_connection_pool(&database_configs).await.expect("Failed to build connection pool");

	sqlx::migrate!("./migrations")
		.run(&db_pool)
		.await
		.expect("Failed to run migrations on new test database");

	db_pool
}