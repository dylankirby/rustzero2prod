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
pub struct SendEmailRequestData {
	text_body: String,
	html_body: String,
	subject: String,
	to: String,
	from: String
}

impl EmailClient {
	pub fn new(base_url: String, sender: SubscriberEmail, authorization_token: String) -> Self {
		Self {
			client: reqwest::Client::new(),
			sender: sender,
			base_url: base_url,
			authorization_token: authorization_token
		}
	}

	pub async fn send_email(&self, recipient: SubscriberEmail, subject: &str, html_content: &str, text_content: &str) -> Result<reqwest::Response, reqwest::Error> {
		let post_data = self.construct_request_json(recipient, &subject, &html_content, &text_content);
		let res = self.client
			.post(&self.construct_url())
			.header("X-Postmark-Server-Token", &self.authorization_token)
			.json(&post_data)
			.send()
			.await?;

		Ok(res)
	}

	pub fn construct_url(&self) -> String {
		return format!("{}/email", self.base_url)
	}

	pub fn construct_request_json(&self, recipient: SubscriberEmail, subject: &str, html_content: &str, text_content: &str) -> SendEmailRequestData {
		SendEmailRequestData {
			text_body: text_content.to_string(),
			html_body: html_content.to_string(),
			subject: subject.to_string(),
			to: recipient.as_ref().to_owned(),
			from: self.sender.as_ref().to_owned()
		}
	}
}

#[cfg(test)]
mod tests {
	use crate::domain::SubscriberEmail;
	use crate::email_client::EmailClient;
	use wiremock::{Match, Request};
	use wiremock::{Mock, MockServer, ResponseTemplate};
	use wiremock::matchers::{header_exists, path, method, header};

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


	#[tokio::test]
	async fn send_email_fires_request_to_base_url() {
		// todo - use fakes for values
		let mock_server = MockServer::start().await;

		let server_uri = mock_server.uri();
		let sender_email = SubscriberEmail::parse("test@test.com".to_string()).expect("failed to parse sender email");
		let auth_token = "AB123".to_string();

		let email_client = EmailClient::new(server_uri, sender_email, auth_token);

		Mock::given(method("POST"))
		.and(path("/email"))
		.and(header_exists("X-Postmark-Server-Token"))
		.and(header("Content-Type", "application/json"))
		.and(SendEmailBodyMatcher)
		.respond_with(ResponseTemplate::new(200))
		.expect(1)
		.mount(&mock_server)
		.await;

		let subject = "Test Email";
		let html_content = "<h1> Test </h1>";
		let content = "Test";
		let recipient = SubscriberEmail::parse("test@gmail.com".to_string()).expect("failed to parse sender email");

		let res = email_client.send_email(recipient, &subject, &html_content, &content)
			.await
			.unwrap();

		assert_eq!(res.status(), 200)
	}
}