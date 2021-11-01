use std::net::TcpListener;
use sqlx::{PgConnection, PgPool, Connection, Executor};
use uuid::Uuid;

use zero2prod::startup::run;
use zero2prod::configurations::{get_configurations, DatabaseSettings};
use zero2prod::telemetry::{get_tracing_subscriber, init_tracing_subscriber};
use zero2prod::email_client::EmailClient;
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

#[actix_rt::test]
async fn health_check_works() {
	let test_app = spawn_app().await;
	let local_uri = format!("{}/health_check", &test_app.address);

	let client = reqwest::Client::new();

	let response = client.get(local_uri)
		.send()
		.await
		.expect("Failed to send request");

	assert!(response.status().is_success());
	assert_eq!(Some(0), response.content_length());
}

#[actix_rt::test]
async fn post_subscribe_returns_200_valid_data() {
	let test_app = spawn_app().await;

	let local_uri = format!("{}/subscriptions", &test_app.address);

	let body = "name=Dylan%20Kirby&email=dk@gmail.com";
	let client = reqwest::Client::new();
	let response = client.post(local_uri)
		.header("Content-Type", "application/x-www-form-urlencoded")
		.body(body)
		.send()
		.await
		.expect("Failed to execute Request");

	assert_eq!(200, response.status().as_u16());

	let saved = sqlx::query!("SELECT email, name FROM subscriptions",)
		.fetch_one(&test_app.db_pool)
		.await
		.expect("Failed to fetch saved subscription");

	
	assert_eq!(saved.email, "dk@gmail.com");
	assert_eq!(saved.name, "Dylan Kirby");

}

#[actix_rt::test]
async fn post_subscribe_returns_400_when_data_missing() {
	let test_app = spawn_app().await;
	let local_uri = format!("{}/subscriptions", &test_app.address);
	let client = reqwest::Client::new();

	let test_cases = vec![
		("name=DylanKirby", "missing email"), 
		("email=dk@dk.com", "missing name"), 
		("", "missing name and email"),
	];

	for (body, description) in test_cases {
		let response = client
			.post(&local_uri)
			.header("Content-Type", "application/x-www-form-urlencoded")
			.body(body)
			.send()
			.await
			.expect("Failed to execute Request");

		assert_eq!(400, response.status().as_u16(), "API did not fail with 400 error code when payload was {}", description);
	}
	
}

#[actix_rt::test]
async fn post_subscribe_returns_400_on_invalid_data() {
	let test_app = spawn_app().await;
	let local_uri = format!("{}/subscriptions", &test_app.address);
	let client = reqwest::Client::new();

	let test_cases = vec![
		("name=&email=jim%40gmail.com", "no name"),
		("name=Jim&email=", "no email"),
		("name=&email=qpp-123-not-an-email", "Invalid Email"),
	];
	
	for (body, description) in test_cases {
		let response = client
			.post(&local_uri)
			.header("Content-Type", "application/x-www-form-urlencoded")
			.body(body)
			.send()
			.await
			.expect("Failed to execute Request");

		assert_eq!(400, response.status().as_u16(), "API did not fail with 400 error code when payload was {}", description);
	}
	
}

async fn spawn_app() -> TestApp {
	Lazy::force(&TRACING);

	let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind to any port");
	
	let port = listener.local_addr().unwrap().port();
	let http_address = format!("http://127.0.0.1:{}", port);

	let mut configs = get_configurations().expect("Unable to load configs");
	let test_db_name = Uuid::new_v4().to_string();
	configs.database.database_name = test_db_name;

	let db_pool = configure_database(&configs.database)
		.await;

	let sender_email = configs.email_client.get_sender_email()
		.expect("Failed to parse sender email, seems invalid");

	let email_client = EmailClient::new(
			configs.email_client.base_url,
			sender_email,
			configs.email_client.authorization_token
		);

	let server = run(listener, db_pool.clone(), email_client).expect("Failed to bind to server to listener");

	let _ = tokio::spawn(server);

	TestApp {
		address: http_address,
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

	let db_pool = PgPool::connect(&database_configs.connection_url())
		.await
		.expect("Failed to generate db poll connection to postgres");

	sqlx::migrate!("./migrations")
		.run(&db_pool)
		.await
		.expect("Failed to run migrations on new test database");

	db_pool
}