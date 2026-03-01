// Type coercion utilities
// Provides CoerceStringToInt for digit-only strings and CoerceOptional for
// Option fields

/// Trait for coercing string to integer types
pub trait CoerceStringToInt: Sized {
	/// Convert a digit-only string to the target integer type
	fn coerce_from_string(s:&str) -> Option<Self>;

	/// Convert an optional string to the target integer type
	fn coerce_from_optional(s:Option<&str>) -> Option<Self>;
}

impl CoerceStringToInt for i32 {
	fn coerce_from_string(s:&str) -> Option<Self> { if is_digit_only(s) { s.parse().ok() } else { None } }

	fn coerce_from_optional(s:Option<&str>) -> Option<Self> { s.and_then(|v| Self::coerce_from_string(v)) }
}

impl CoerceStringToInt for i64 {
	fn coerce_from_string(s:&str) -> Option<Self> { if is_digit_only(s) { s.parse().ok() } else { None } }

	fn coerce_from_optional(s:Option<&str>) -> Option<Self> { s.and_then(|v| Self::coerce_from_string(v)) }
}

impl CoerceStringToInt for u32 {
	fn coerce_from_string(s:&str) -> Option<Self> { if is_digit_only(s) { s.parse().ok() } else { None } }

	fn coerce_from_optional(s:Option<&str>) -> Option<Self> { s.and_then(|v| Self::coerce_from_string(v)) }
}

impl CoerceStringToInt for u64 {
	fn coerce_from_string(s:&str) -> Option<Self> { if is_digit_only(s) { s.parse().ok() } else { None } }

	fn coerce_from_optional(s:Option<&str>) -> Option<Self> { s.and_then(|v| Self::coerce_from_string(v)) }
}

/// Check if a string contains only ASCII digits
pub fn is_digit_only(s:&str) -> bool { !s.is_empty() && s.chars().all(|c| c.is_ascii_digit()) }

/// Coerce a string to an integer, returning None if not digit-only
pub fn coerce_string_to_int<T:CoerceStringToInt>(s:&str) -> Option<T> { T::coerce_from_string(s) }

/// Coerce an optional string to an integer
pub fn coerce_optional_string_to_int<T:CoerceStringToInt>(s:Option<String>) -> Option<T> {
	s.and_then(|v| T::coerce_from_string(v.as_str()))
}

/// Trait for coercing Option fields
pub trait CoerceOptional<T> {
	/// Coerce an Option<String> to Option<T>
	fn coerce(opt:Option<String>) -> Option<T>;
}

impl<T:CoerceStringToInt> CoerceOptional<T> for T {
	fn coerce(opt:Option<String>) -> Option<T> { opt.and_then(|v| T::coerce_from_string(v.as_str())) }
}

/// Coerce a string field that might be numeric or string
/// Returns the integer value if the string contains only digits, otherwise None
pub fn coerce_numeric_string(s:&str) -> Option<i64> {
	let trimmed = s.trim();
	if trimmed.is_empty() {
		None
	} else if is_digit_only(trimmed) {
		trimmed.parse().ok()
	} else {
		None
	}
}

/// Coerce a string field to i32
pub fn coerce_to_i32(s:&str) -> Option<i32> { coerce_string_to_int(s) }

/// Coerce a string field to i64
pub fn coerce_to_i64(s:&str) -> Option<i64> { coerce_string_to_int(s) }

/// Coerce a string field to u32
pub fn coerce_to_u32(s:&str) -> Option<u32> { coerce_string_to_int(s) }

/// Coerce a string field to u64
pub fn coerce_to_u64(s:&str) -> Option<u64> { coerce_string_to_int(s) }

/// Coerce an optional string to i32
pub fn coerce_optional_to_i32(s:Option<String>) -> Option<i32> { coerce_optional_string_to_int(s) }

/// Coerce an optional string to i64
pub fn coerce_optional_to_i64(s:Option<String>) -> Option<i64> { coerce_optional_string_to_int(s) }

/// Coerce an optional string to u32
pub fn coerce_optional_to_u32(s:Option<String>) -> Option<u32> { coerce_optional_string_to_int(s) }

/// Coerce an optional string to u64
pub fn coerce_optional_to_u64(s:Option<String>) -> Option<u64> { coerce_optional_string_to_int(s) }

/// Coerce a boolean from various representations
pub fn coerce_to_bool(s:&str) -> Option<bool> {
	match s.to_lowercase().as_str() {
		"true" | "1" | "yes" | "on" => Some(true),
		"false" | "0" | "no" | "off" => Some(false),
		_ => None,
	}
}

/// Coerce an optional boolean
pub fn coerce_optional_to_bool(s:Option<String>) -> Option<bool> { s.and_then(|v| coerce_to_bool(&v)) }

/// Coerce a string, trimming whitespace
pub fn coerce_trim(s:&str) -> String { s.trim().to_string() }

/// Coerce an optional string, returning None for empty strings
pub fn coerce_optional_trim(s:Option<String>) -> Option<String> {
	s.map(|v| v.trim().to_string()).filter(|v| !v.is_empty())
}

/// Coerce an optional string to lowercase
pub fn coerce_to_lowercase(s:&str) -> String { s.to_lowercase() }

/// Coerce an optional to lowercase and trim
pub fn coerce_optional_to_lowercase(s:Option<String>) -> Option<String> {
	s.map(|v| v.trim().to_lowercase()).filter(|v| !v.is_empty())
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_is_digit_only() {
		assert!(is_digit_only("123"));
		assert!(is_digit_only("0"));
		assert!(!is_digit_only("12a"));
		assert!(!is_digit_only(""));
		assert!(!is_digit_only("12.3"));
	}

	#[test]
	fn test_coerce_string_to_int() {
		assert_eq!(coerce_to_i32("123"), Some(123));
		assert_eq!(coerce_to_i32("abc"), None);
		assert_eq!(coerce_to_i64("1234567890123"), Some(1234567890123));
	}

	#[test]
	fn test_coerce_optional() {
		assert_eq!(coerce_optional_to_i32(Some("123".to_string())), Some(123));
		assert_eq!(coerce_optional_to_i32(Some("".to_string())), None);
		assert_eq!(coerce_optional_to_i32(None), None);
	}

	#[test]
	fn test_coerce_to_bool() {
		assert_eq!(coerce_to_bool("true"), Some(true));
		assert_eq!(coerce_to_bool("1"), Some(true));
		assert_eq!(coerce_to_bool("false"), Some(false));
		assert_eq!(coerce_to_bool("0"), Some(false));
		assert_eq!(coerce_to_bool("invalid"), None);
	}
}
