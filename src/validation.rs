use unicode_segmentation::UnicodeSegmentation;
use validator::validate_email;

const FORBIDDEN_NAME_CHARS: [char; 9] = ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];

pub fn is_valid_name(name: &str) -> bool{
	let is_empty_or_whitespace = name.trim().is_empty();
	let is_too_long = name.graphemes(true).count() > 256;
	let contains_forbidden_char = name.chars().any(|g| FORBIDDEN_NAME_CHARS.contains(&g));
	!(is_empty_or_whitespace || is_too_long || contains_forbidden_char) 
}

pub fn is_valid_email(email: &str) -> bool {
	validate_email(email)
}

#[cfg(test)]
mod tests {
	use crate::validation::{is_valid_name, is_valid_email};

	#[test]
	fn name_longer_than_256_is_rejected() {
		let name = "a".repeat(257);
		let res = is_valid_name(&name);
		assert_eq!(res, false);
	}


	#[test]
	fn empty_name_is_rejected() {
		let name = "";
		let res = is_valid_name(&name);
		assert_eq!(res, false);
	}


	#[test]
	fn name_with_only_whitespace_is_rejected() {
		let name = "    ";
		let res = is_valid_name(&name);
		assert_eq!(res, false);
	}


	#[test]
	fn empty_email_is_rejected() {
		let email = "";
		let res = is_valid_email(&email);
		assert_eq!(res, false);
	}


	#[test]
	fn email_with_only_whitespace_is_rejected() {
		let email = "    ";
		let res = is_valid_email(&email);
		assert_eq!(res, false);
	}

	#[test]
	fn email_without_at_symbol_is_rejected() {
		let email = "somedomain.com";
		let res = is_valid_email(&email);
		assert_eq!(res, false);
	}

	#[test]
	fn email_without_subject_is_rejected() {
		let email = "@gmail.com";
		let res = is_valid_email(&email);
		assert_eq!(res, false);
	}

	#[test]
	fn name_with_invalid_charaters_is_rejected() {
		for name in &['/', '(', ')', '"', '<', '>', '\\', '{', '}'] {
			let name = name.to_string();
			let res = is_valid_name(&name);
			assert_eq!(res, false);
		}
	}	
}