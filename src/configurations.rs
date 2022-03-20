use serde::Deserialize;
use std::convert::{TryFrom, TryInto};

use crate::domain::SubscriberEmail;

#[derive(Deserialize)]
#[derive(Clone)]
pub struct Settings {
	pub database: DatabaseSettings,
	pub application: ApplicationSettings,
	pub email_client: EmailClientSettings
}

// application settings
#[derive(Deserialize)]
#[derive(Clone)]
pub struct ApplicationSettings {
	pub host: String,
	pub port: u16
}

// database settings
#[derive(Deserialize)]
#[derive(Clone)]
pub struct DatabaseSettings {
	pub username: String,
	pub password: String,
	pub port: u16,
	pub host: String,
	pub database_name: String
}

impl DatabaseSettings {
	pub fn connection_url(&self) -> String {
		format!(
			"postgres://{}:{}@{}:{}/{}",
			self.username,
			self.password,
			self.host,
			self.port,
			self.database_name
		)
	}

	pub fn connection_url_without_db(&self) -> String {
		format!(
			"postgres://{}:{}@{}:{}",
			self.username,
			self.password,
			self.host,
			self.port
		)
	}
}

// email client settings
#[derive(Deserialize)]
#[derive(Clone)]
pub struct EmailClientSettings {
	pub base_url: String,
	pub sender_email: String,
	pub authorization_token: String,
	pub timeout_ms: u64
}

impl EmailClientSettings {
	pub fn get_sender_email(&self) -> Result<SubscriberEmail, String> {
		SubscriberEmail::parse(self.sender_email.clone())
	}

	pub fn timeout(&self) -> std::time::Duration {
		std::time::Duration::from_millis(self.timeout_ms)
	}
}

// env configurations
pub enum Environment {
	Local,
	Production
}

impl Environment {
	pub fn as_str(&self) -> &'static str {
		match self {
			Environment::Local => "local",
			Environment::Production => "production"
		}
	}
}

impl TryFrom<String> for Environment {
	type Error = String;

	fn try_from(s: String) -> Result<Self, Self::Error> {
		match s.to_lowercase().as_str() {
			"local" => Ok(Self::Local),
			"production" => Ok(Self::Production),
			other => Err(format!("{} is not a supported environment", other))
		}
	}
}

// helper methods
pub fn get_configurations() -> Result<Settings, config::ConfigError> {
	// Init config module's configuration reader
	let mut settings = config::Config::default();
	let base_path = std::env::current_dir().expect("Failed to determin current directory");
	let config_directory = base_path.join("configuration");
	settings.merge(config::File::from(config_directory.join("base")).required(true))?;

	let environment: Environment = std::env::var("APP_ENVIRONMENT")
		.unwrap_or_else(|_| "local".into())
		.try_into()
		.expect("Failed to parse environment");

	settings.merge(config::File::from(config_directory.join(environment.as_str())).required(true))?;
	settings.merge(config::Environment::with_prefix("app").separator("__"))?;


	//tries to conver the settings reader values into values that fit into our
	//Settings struct
	settings.try_into()
}