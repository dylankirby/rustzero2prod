use uuid::Uuid;
use chrono::Utc;
use std::convert::TryInto;

use sqlx::{PgPool};

use actix_web::{web, HttpResponse};

use crate::domain::{SubscriberDetails, SubscriptionFormData, SubscriberEmail};
use crate::email_client::EmailClient;

const INVITED_STATUS: &str = "invited";


#[tracing::instrument(
	name = "Adding new subscriber",
	skip(form, db_pool),
	fields(
		subscriber_email = %form.email,
		subscriber_name = %form.name
	)
)]
pub async fn subscriptions_post(form: web::Form<SubscriptionFormData>, db_pool: web::Data<PgPool>, email_client: web::Data<EmailClient>) -> HttpResponse {
	let subscriber_details = match form.0.try_into() {
		Ok(subscriber_details) => subscriber_details,
		Err(_) => return HttpResponse::BadRequest().finish()
	};

	let subscriber_id = Uuid::new_v4();
	
	if insert_subscriber(subscriber_id.clone(), &subscriber_details, &db_pool).await.is_err() {
		return HttpResponse::InternalServerError().finish()
	};

	if send_new_subscriber_email(subscriber_id.clone(), subscriber_details.email, &email_client).await.is_err() {
		return HttpResponse::InternalServerError().finish()
	}

	HttpResponse::Ok().finish()

}

#[tracing::instrument(
	name = "Saving new subscriber to database",
	skip(new_subscriber, db_pool)
)]
pub async fn insert_subscriber(subscriber_id: Uuid, new_subscriber: &SubscriberDetails, db_pool: &PgPool) -> Result<(), sqlx::Error>{
	sqlx::query!(
		r#"
			INSERT INTO subscriptions (id, email, name, subscribed_at, status)
			VALUES ($1, $2, $3, $4, $5)
		"#,
		subscriber_id,
		new_subscriber.email.as_ref(),
		new_subscriber.name.as_ref(),
		Utc::now(),
		INVITED_STATUS
	)
	.execute(db_pool) // Attach the query span to the query to attach tracing to the request future
	.await
	.map_err(|e| {
		tracing::error!("Failed to execute SQL Insert due to: {:?}", e);
		e
	})?;
	Ok(())
}

#[tracing::instrument(
	name = "Sending Subscirber confirmation Email",
)]
pub async fn send_new_subscriber_email(new_subscriber_uuid: Uuid, subscriber_email: SubscriberEmail, email_client: &EmailClient) -> Result<(), reqwest::Error> {
	let subject = "Hello";
	let html_content = "<h1> Hello </h1>";
	let content = "Hello";
	email_client.send_email(subscriber_email, subject, html_content, content).await?;
	Ok(())
}