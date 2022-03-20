use std::net::TcpListener;

use sqlx::{PgPool};
use sqlx::postgres::PgPoolOptions;
use actix_web::dev::Server;
use actix_web::{web, App, HttpServer};
use actix_web::web::Data;
use tracing_actix_web::TracingLogger;

use crate::configurations::{Settings, DatabaseSettings};
use crate::email_client::EmailClient;
use crate::routes::{health_check, subscriptions_post};

pub struct Application {
    port: u16,
    server: Server
}

impl Application {
    pub async fn build(configs: Settings) -> Result<Self, std::io::Error> {
        let db_pool = build_connection_pool(&configs.database)
            .await
            .expect("Failed ot build PGPool");

        let sender_email = configs.email_client.get_sender_email()
            .expect("Failed to parse sender email, seems invalid");


        let email_client_timeout = configs.email_client.timeout();
        let email_client = EmailClient::new(
            configs.email_client.base_url,
            sender_email,
            configs.email_client.authorization_token,
            email_client_timeout
        );

        let application_address = format!("{}:{}", configs.application.host, configs.application.port);
        let listener = TcpListener::bind(&application_address)?;
        let port = listener.local_addr().unwrap().port();

        let server = run(listener, db_pool, email_client)?;
        Ok(Self {port, server})
    }

    pub fn port(&self) -> u16 { 
        self.port
    }

    pub async fn run_server(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}

pub fn run(listener: TcpListener, db_pool: PgPool, email_client: EmailClient) -> Result<Server, std::io::Error> {
	let app_db_pool = Data::new(db_pool);
    let app_email_client = Data::new(email_client);
	let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .route("/health_check", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscriptions_post))
            .app_data(app_db_pool.clone())
            .app_data(app_email_client.clone())
    })
    .listen(listener)?
    .run();

    Ok(server)
}

pub async fn build_connection_pool(database_configs: &DatabaseSettings) -> Result<PgPool, sqlx::Error> {
    let db_connection_url = database_configs.connection_url();
    PgPoolOptions::new()
        .connect_timeout(std::time::Duration::from_secs(2))
        .connect(&db_connection_url)
        .await
}