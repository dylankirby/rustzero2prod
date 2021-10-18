use uuid::Uuid;
use chrono::Utc;
use std::convert::TryInto;

use sqlx::{PgPool};

use actix_web::{web, HttpResponse};

use crate::domain::{SubscriberDetails, SubscriptionFormData};




#[tracing::instrument(
	name = "Adding new subscriber",
	skip(form, db_pool),
	fields(
		subscriber_email = %form.email,
		subscriber_name = %form.name
	)
)]
pub async fn subscriptions_post(form: web::Form<SubscriptionFormData>, db_pool: web::Data<PgPool>) -> HttpResponse {
	let subscriber_deails = match form.0.try_into() {
		Ok(subscriber_deails) => subscriber_deails,
		Err(_) => return HttpResponse::BadRequest().finish()
	};
	match insert_subscriber(&subscriber_deails, &db_pool).await {
		Ok(_) => HttpResponse::Ok().finish(),
		Err(_) => HttpResponse::InternalServerError().finish()
	}
}

#[tracing::instrument(
	name = "Saving new subscriber to database",
	skip(new_subscriber, db_pool)
)]
pub async fn insert_subscriber(new_subscriber: &SubscriberDetails, db_pool: &PgPool) -> Result<(), sqlx::Error>{
	let sub_uuid = Uuid::new_v4();
	sqlx::query!(
		r#"
			INSERT INTO subscriptions (id, email, name, subscribed_at)
			VALUES ($1, $2, $3, $4)
		"#,
		sub_uuid,
		new_subscriber.email.as_ref(),
		new_subscriber.name.as_ref(),
		Utc::now()
	)
	.execute(db_pool) // Attach the query span to the query to attach tracing to the request future
	.await
	.map_err(|e| {
		tracing::error!("Failed to execute SQL Insert due to: {:?}", e);
		e
	})?;
	Ok(())
}