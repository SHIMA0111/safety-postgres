use crate::postgres::errors::ErrorGenerator;

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
///
/// # Examples
///
/// ```
/// let is_valid = validate_alphanumeric_name("Abc_123", "_");
/// assert_eq!(is_valid, true);
///
/// let is_valid = validate_alphanumeric_name("Hello@123", "_");
/// assert_eq!(is_valid, false);
/// ```
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