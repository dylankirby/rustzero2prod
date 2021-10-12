use unicode_segmentation::UnicodeSegmentation;

const FORBIDDEN_NAME_CHARS: [char; 9] = ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];

pub fn is_valid_name(name: &str) -> bool{
	let is_empty_or_whitespace = name.trim().is_empty();
	let is_too_long = name.graphemes(true).count() > 256;
	let contains_forbidden_char = name.chars().any(|g| FORBIDDEN_NAME_CHARS.contains(&g));
	!(is_empty_or_whitespace || is_too_long || contains_forbidden_char) 
}

pub fn is_valid_email(email: &str) -> bool {
	let is_empty_or_whitespace = email.trim().is_empty();

	return !(is_empty_or_whitespace)
}