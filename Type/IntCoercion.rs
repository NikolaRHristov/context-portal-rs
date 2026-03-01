// Integer coercion for string fields
// Provides CoerceInt() trait for digit-only string to integer conversion
// Includes Deserialize implementations for i32, i64, u32, u64

use std::fmt;

use serde::de::{self, Deserialize, Deserializer, Visitor};

/// Trait for coercing digit-only strings to integers
pub trait CoerceInt: Sized {
	type Output;

	/// Try to coerce a string to an integer if it contains only digits
	fn coerce_int(&self) -> Option<Self::Output>;
}

/// CoerceInt implementation for String
impl CoerceInt for String {
	type Output = i64;

	fn coerce_int(&self) -> Option<Self::Output> {
		if self.chars().all(|c| c.is_ascii_digit()) { self.parse().ok() } else { None }
	}
}

/// CoerceInt implementation for &str
impl CoerceInt for &str {
	type Output = i64;

	fn coerce_int(&self) -> Option<Self::Output> {
		if self.chars().all(|c| c.is_ascii_digit()) { self.parse().ok() } else { None }
	}
}

/// Check if a string contains only digits
pub fn is_digit_only(s:&str) -> bool { !s.is_empty() && s.chars().all(|c| c.is_ascii_digit()) }

/// Coerce string to i32 if it contains only digits
pub fn coerce_to_i32(s:&str) -> Option<i32> { if is_digit_only(s) { s.parse().ok() } else { None } }

/// Coerce string to i64 if it contains only digits
pub fn coerce_to_i64(s:&str) -> Option<i64> { if is_digit_only(s) { s.parse().ok() } else { None } }

/// Coerce string to u32 if it contains only digits
pub fn coerce_to_u32(s:&str) -> Option<u32> { if is_digit_only(s) { s.parse().ok() } else { None } }

/// Coerce string to u64 if it contains only digits
pub fn coerce_to_u64(s:&str) -> Option<u64> { if is_digit_only(s) { s.parse().ok() } else { None } }

/// Visitor for deserializing integers that can come as either strings or
/// numbers
struct IntVisitor<T> {
	_phantom:std::marker::PhantomData<T>,
}

impl<T> IntVisitor<T> {
	fn new() -> Self { Self { _phantom:std::marker::PhantomData } }
}

impl<'de, T> Visitor<'de> for IntVisitor<T>
where
	T: Deserialize<'de> + TryFrom<i64> + TryFrom<String>,
	<T as TryFrom<i64>>::Error: fmt::Debug,
	<T as TryFrom<String>>::Error: fmt::Debug,
{
	type Value = T;

	fn expecting(&self, formatter:&mut fmt::Formatter) -> fmt::Result {
		formatter.write_str("an integer or a digit-only string")
	}

	fn visit_i64<E>(self, value:i64) -> Result<Self::Value, E>
	where
		E: de::Error, {
		T::try_from(value).map_err(|_| de::Error::custom("integer overflow"))
	}

	fn visit_u64<E>(self, value:u64) -> Result<Self::Value, E>
	where
		E: de::Error, {
		T::try_from(value as i64).map_err(|_| de::Error::custom("integer overflow"))
	}

	fn visit_str<E>(self, value:&str) -> Result<Self::Value, E>
	where
		E: de::Error, {
		if is_digit_only(value) {
			T::try_from(value.to_string()).map_err(|_| de::Error::custom("invalid integer string"))
		} else {
			Err(de::Error::custom("string is not digit-only"))
		}
	}

	fn visit_string<E>(self, value:String) -> Result<Self::Value, E>
	where
		E: de::Error, {
		if is_digit_only(&value) {
			T::try_from(value).map_err(|_| de::Error::custom("invalid integer string"))
		} else {
			Err(de::Error::custom("string is not digit-only"))
		}
	}
}

/// Deserialize an i32 from either a number or a digit-only string
pub fn deserialize_i32<'de, D>(deserializer:D) -> Result<i32, D::Error>
where
	D: Deserializer<'de>, {
	deserializer.deserialize_any(IntVisitor::<i32>::new())
}

/// Deserialize an i64 from either a number or a digit-only string
pub fn deserialize_i64<'de, D>(deserializer:D) -> Result<i64, D::Error>
where
	D: Deserializer<'de>, {
	deserializer.deserialize_any(IntVisitor::<i64>::new())
}

/// Deserialize a u32 from either a number or a digit-only string
pub fn deserialize_u32<'de, D>(deserializer:D) -> Result<u32, D::Error>
where
	D: Deserializer<'de>, {
	deserializer.deserialize_any(IntVisitor::<u32>::new())
}

/// Deserialize a u64 from either a number or a digit-only string
pub fn deserialize_u64<'de, D>(deserializer:D) -> Result<u64, D::Error>
where
	D: Deserializer<'de>, {
	deserializer.deserialize_any(IntVisitor::<u64>::new())
}

/// Helper for optional field coercion - returns None if string is empty or not
/// digit-only
pub fn coerce_optional_i32(s:Option<String>) -> Option<i32> {
	s.and_then(|v| {
		let trimmed = v.trim();
		if trimmed.is_empty() { None } else { coerce_to_i32(trimmed) }
	})
}

/// Helper for optional field coercion - returns None if string is empty or not
/// digit-only
pub fn coerce_optional_i64(s:Option<String>) -> Option<i64> {
	s.and_then(|v| {
		let trimmed = v.trim();
		if trimmed.is_empty() { None } else { coerce_to_i64(trimmed) }
	})
}

/// Helper for optional field coercion - returns None if string is empty or not
/// digit-only
pub fn coerce_optional_u32(s:Option<String>) -> Option<u32> {
	s.and_then(|v| {
		let trimmed = v.trim();
		if trimmed.is_empty() { None } else { coerce_to_u32(trimmed) }
	})
}

/// Helper for optional field coercion - returns None if string is empty or not
/// digit-only
pub fn coerce_optional_u64(s:Option<String>) -> Option<u64> {
	s.and_then(|v| {
		let trimmed = v.trim();
		if trimmed.is_empty() { None } else { coerce_to_u64(trimmed) }
	})
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
		assert!(!is_digit_only("-123"));
	}

	#[test]
	fn test_coerce_to_i32() {
		assert_eq!(coerce_to_i32("123"), Some(123));
		assert_eq!(coerce_to_i32("0"), Some(0));
		assert_eq!(coerce_to_i32("abc"), None);
		assert_eq!(coerce_to_i32(""), None);
	}

	#[test]
	fn test_coerce_optional() {
		assert_eq!(coerce_optional_i32(Some("123".to_string())), Some(123));
		assert_eq!(coerce_optional_i32(Some("".to_string())), None);
		assert_eq!(coerce_optional_i32(None), None);
	}
}
