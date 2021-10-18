use std::convert::TryInto;
use serde::Deserialize;

use crate::validation::{is_valid_name, is_valid_email};

#[derive(Deserialize)]
pub struct SubscriptionFormData {
    pub name: String,
    pub email: String
}

#[derive(Debug)]
pub struct SubscriberDetails {
	pub name: SubscriberName,
	pub email: SubscriberEmail,
}

impl TryInto<SubscriberDetails> for SubscriptionFormData {
	type Error = String;

	fn try_into(self) -> Result<SubscriberDetails, Self::Error> {
		let name = SubscriberName::parse(self.name)?;
		let email = SubscriberEmail::parse(self.email)?;

		Ok(SubscriberDetails {
			name: name,
			email: email
		})
	}
}

#[derive(Debug)]
pub struct SubscriberName(String);


impl SubscriberName {
	pub fn parse(s: String) -> Result<SubscriberName, String> {
		if !is_valid_name(&s) {
			Err(format!("{} failed name validation", s))
		} else {
			Ok(Self(s))
		}
	}
}

#[derive(Debug)]
pub struct SubscriberEmail(String);

impl SubscriberEmail {
	pub fn parse(s: String) -> Result<SubscriberEmail, String> {
		if !is_valid_email(&s) {
			Err(format!("{} failed email validation", s))
		} else {
			Ok(Self(s))
		}
	}
}

macro_rules! impl_AsRef_for_Subscriber_fields {
	(for $($t:ty),+) => {
		$(impl AsRef<str> for $t {
			fn as_ref(&self) -> &str {
				&self.0
			}
		})*
	};
}

impl_AsRef_for_Subscriber_fields!(for SubscriberName, SubscriberEmail);


#[cfg(test)]
mod subscriber_name_tests {
	use crate::domain::SubscriberName;
	use claim::{assert_err, assert_ok};

	#[test]
	fn test_parse_invalid_name_raises_err() {
		let invalid_name = "a".repeat(257);
		assert_err!(SubscriberName::parse(invalid_name));
	}


	#[test]
	fn test_parse_vali_name_returns_ok() {
		let valid_name = "a".repeat(25);
		assert_ok!(SubscriberName::parse(valid_name));
	}
}

#[cfg(test)]
mod subscriber_email_tests {
	use fake::Fake;
	use fake::faker::internet::en::SafeEmail;
	use crate::domain::SubscriberEmail;
	use claim::{assert_err, assert_ok};

	#[test]
	fn test_parse_invalid_email_raises_err() {
		let invalid_email = "      ".to_string();
		assert_err!(SubscriberEmail::parse(invalid_email));
	}

	#[test]
	fn test_parse_valid_email_returns_ok() {
		let valid_email = "dylan@gmail.com".to_string();
		assert_ok!(SubscriberEmail::parse(valid_email));
	}

	#[test]
	fn test_parse_valid_email_is_successful() {
		let email = SafeEmail().fake();
		assert_ok!(SubscriberEmail::parse(email));
	}
}