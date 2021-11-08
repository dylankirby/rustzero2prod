use std::net::TcpListener;

use sqlx::{PgPool};
use sqlx::postgres::PgPoolOptions;
use actix_web::dev::Server;
use actix_web::{web, App, HttpServer};
use actix_web::web::Data;
use tracing_actix_web::TracingLogger;

use crate::configurations::Settings;
use crate::email_client::EmailClient;
use crate::routes::{health_check, subscriptions_post};


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

pub async fn pre_run_build(configs: Settings) -> Result<(TcpListener, EmailClient, PgPool), std::io::Error>  {
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

    return Ok((listener, email_client, db_pool))
}
