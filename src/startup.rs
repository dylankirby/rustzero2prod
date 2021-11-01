use std::net::TcpListener;

use sqlx::{PgPool};
use actix_web::dev::Server;
use actix_web::{web, App, HttpServer};
use actix_web::web::Data;
use tracing_actix_web::TracingLogger;

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
