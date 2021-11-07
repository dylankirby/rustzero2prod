use crate::helpers::spawn_app;

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