use serde;
use reqwest;
use crate::domain::SubscriberEmail;


pub struct EmailClient {
	sender: SubscriberEmail,
	client: reqwest::Client,
	base_url: String,
	authorization_token: String
}

#[derive(serde::Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct SendEmailRequestData<'a> {
	text_body: &'a str,
	html_body: &'a str,
	subject: &'a str,
	to: &'a str,
	from: &'a str
}

impl EmailClient {
	pub fn new(base_url: String, sender: SubscriberEmail, authorization_token: String) -> Self {
		let http_client = reqwest::Client::builder()
			.timeout(std::time::Duration::from_secs(10))
			.build()
			.unwrap();

		Self {
			client: http_client,
			sender: sender,
			base_url: base_url,
			authorization_token: authorization_token
		}
	}

	pub async fn send_email(&self, recipient: SubscriberEmail, subject: &str, html_content: &str, text_content: &str) -> Result<(), reqwest::Error> {
		let request_body = SendEmailRequestData {
			text_body: text_content,
			html_body: html_content,
			subject: subject,
			to: recipient.as_ref(),
			from: self.sender.as_ref()
		};

		let url = self.construct_url();

		self.client
			.post(&url)
			.header("X-Postmark-Server-Token", &self.authorization_token)
			.json(&request_body)
			.send()
			.await?
			.error_for_status()?;

		Ok(())
	}

	pub fn construct_url(&self) -> String {
		return format!("{}/email", self.base_url)
	}

}

#[cfg(test)]
mod tests {
	use crate::domain::SubscriberEmail;
	use crate::email_client::EmailClient;

	use fake::faker::internet::en::SafeEmail;
	use fake::faker::lorem::en::{Paragraph, Sentence};
	use fake::{Fake, Faker};

	use wiremock::{Match, Request};
	use wiremock::{Mock, MockServer, ResponseTemplate};
	use wiremock::matchers::{header_exists, path, method, header, any};
	use claim::{assert_ok, assert_err};

	// Define custom matcher for expected body
	struct SendEmailBodyMatcher;

	impl Match for SendEmailBodyMatcher {
		fn matches(&self, request: &Request) -> bool {
			let res: Result<serde_json::Value, _> = serde_json::from_slice(&request.body);
			if let Ok(json_body) = res {
				json_body.get("TextBody").is_some() && json_body.get("HtmlBody").is_some() && json_body.get("Subject").is_some() && json_body.get("To").is_some() && json_body.get("From").is_some()
			} else {
				return false;
			}
		}
	}

	fn sender_email() -> SubscriberEmail {
		SubscriberEmail::parse("test@test.com".to_string()).expect("failed to parse sender email")
	}

	fn auth_token() -> String {
		"AB123".to_string()
	}

	fn email_client(server_uri: String) -> EmailClient {
		EmailClient::new(server_uri, sender_email(), auth_token())
	}

	fn subject() -> String {
		Paragraph(1..2).fake()
	}

	fn html_content() -> String {
		content()
	}

	fn content() -> String {
		Paragraph(1..10).fake()
	}

	fn recipient() -> SubscriberEmail {
		SubscriberEmail::parse(SafeEmail().fake()).expect("failed to parse sender email")
	}



	#[tokio::test]
	async fn send_email_fires_request_to_base_url() {
		let mock_server = MockServer::start().await;
		let server_uri = mock_server.uri();

		let email_client = email_client(server_uri);

		Mock::given(method("POST"))
		.and(path("/email"))
		.and(header_exists("X-Postmark-Server-Token"))
		.and(header("Content-Type", "application/json"))
		.and(SendEmailBodyMatcher)
		.respond_with(ResponseTemplate::new(200))
		.expect(1)
		.mount(&mock_server)
		.await;

		let _ = email_client.send_email(recipient(), &subject(), &html_content(), &content())
			.await;
	}

	#[tokio::test]
	async fn send_email_succeeds_on_200_return() {
		let mock_server = MockServer::start().await;
		let server_uri = mock_server.uri();

		let email_client = email_client(server_uri);

		Mock::given(any())
		.respond_with(ResponseTemplate::new(200))
		.expect(1)
		.mount(&mock_server)
		.await;


		let res = email_client.send_email(recipient(), &subject(), &html_content(), &content())
			.await;

		assert_ok!(res)
	}

	#[tokio::test]
	async fn send_email_failed_on_500() {
		let mock_server = MockServer::start().await;
		let server_uri = mock_server.uri();

		let email_client = email_client(server_uri);

		Mock::given(any())
		.respond_with(ResponseTemplate::new(500))
		.expect(1)
		.mount(&mock_server)
		.await;

		let res = email_client.send_email(recipient(), &subject(), &html_content(), &content())
			.await;

		assert_err!(res);
	}

	#[tokio::test]
	async fn send_email_on_timeout() {
		let mock_server = MockServer::start().await;
		let server_uri = mock_server.uri();

		let email_client = email_client(server_uri);

		let response = ResponseTemplate::new(200)
			.set_delay(std::time::Duration::from_secs(180));

		Mock::given(any())
		.respond_with(response)
		.expect(1)
		.mount(&mock_server)
		.await;

		let res = email_client.send_email(recipient(), &subject(), &html_content(), &content())
			.await;

		assert_err!(res);
	}
}