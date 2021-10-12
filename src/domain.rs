use serde::Deserialize;

use crate::validation::{is_valid_name, is_valid_email};

#[derive(Deserialize)]
pub struct SubscriptionFormData {
    pub name: String,
    pub email: String
}

pub struct SubscriberDetails {
	pub name: SubscriberName,
	pub email: SubscriberEmail,
}

impl SubscriberDetails {
	pub fn from_form(form: &SubscriptionFormData) -> SubscriberDetails{
		let name = SubscriberName(form.name.clone());
		let email = SubscriberEmail(form.email.clone());

		Self {
			name: name,
			email: email
		}
	}
}

pub struct SubscriberName(String);


impl SubscriberName {
	pub fn parse(s: String) -> SubscriberName{
		if is_valid_name(&s) {
			Self(s)
		} else {
			panic!("{} failed name validation", s)
		}
	}
}

pub struct SubscriberEmail(String);

impl SubscriberEmail {
	pub fn parse(s: String) -> SubscriberEmail {
		if is_valid_email(&s) {
			Self(s)
		} else {
			panic!("{} failed email validation", s)
		}
	}
}

macro_rules! impl_as_ref_for_Subscriber_fields {
	(for $($t:ty),+) => {
		$(impl AsRef<str> for $t {
			fn as_ref(&self) -> &str {
				&self.0
			}
		})*
	};
}

impl_as_ref_for_Subscriber_fields!(for SubscriberName, SubscriberEmail);