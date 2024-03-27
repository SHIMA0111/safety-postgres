use crate::legacies::errors::ErrorGenerator;

/// Validates if a string contains only alphanumeric characters or characters from a provided allow list.
///
/// # Arguments
///
/// * `s` - The string to be validated.
/// * `allow_chars` - A string containing characters that are allowed in addition to alphanumeric characters.
///
/// # Returns
///
/// Returns `true` if the string is valid, otherwise `false`.
pub(super) fn validate_alphanumeric_name(s: &str, allow_chars: &str) -> bool {
    s.chars().all(|char| char.is_alphanumeric() || allow_chars.contains(char))
}

/// Validates a string based on a specific criteria.
/// If the string contains invalid characters, an error is returned.
///
/// # Arguments
///
/// * `str` - The string to be validated.
/// * `param_name` - The name of the parameter the string represents.
/// * `error_generator` - An instance of a type that implements the `ErrorGenerator` trait,
///                       which is used to generate the appropriate error.
///
/// # Returns
///
/// Returns `Ok(())` if the string is valid, otherwise returns an error.
pub(super) fn validate_string<E, G>(str: &str, param_name: &str, error_generator: &G) -> Result<(), E> where G: ErrorGenerator<E> {
    if !validate_alphanumeric_name(str, "_") {
        let error_message = format!("'{}' has invalid characters. '{}' allows alphabets, numbers and under bar only.", str, param_name);
        return Err(error_generator.generate_error(error_message));
    } else {
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use crate::legacies::errors::{JoinTableError, JoinTableErrorGenerator};
    use super::*;

    /// Tests `validate_alphanumeric_name` function.
    /// It checks if the function correctly validates whether given string consists of only alphanumeric characters and allowed characters,
    /// and correctly returns `false` for invalid strings and `true` for valid strings.
    #[test]
    fn test_valid_alphanumeric_name() {
        let symbols_invalid = "`~!@#$%^&*()+=-{}|:\"<>?[]\\;',./";
        for symbol_char in symbols_invalid.chars() {
            assert_eq!(validate_alphanumeric_name(format!("Abc{}1098", symbol_char).as_str(), "_"), false);
        }

        assert_eq!(validate_alphanumeric_name("abD_234", "_"), true);
    }

    /// Tests that `validate_string` function correctly validates given string and parameter name,
    /// and uses the provided error generator to generate the appropriate error for invalid strings.
    #[test]
    fn test_valid_string() {
        let valid_text = "aBc_123";
        let invalid_text = "aBc@123";

        assert_eq!(validate_string(valid_text, "test1", &JoinTableErrorGenerator), Ok(()));
        assert_eq!(validate_string(invalid_text, "test2", &JoinTableErrorGenerator),
                   Err(JoinTableError::InputInvalidError(format!("'{}' has invalid characters. '{}' allows alphabets, numbers and under bar only.", invalid_text, "test2"))));
    }
}
