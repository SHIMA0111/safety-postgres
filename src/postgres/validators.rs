use crate::postgres::errors::ErrorGenerator;

pub(super) fn validate_alphanumeric_name(s: &str, allow_chars: &str) -> bool {
    s.chars().all(|char| char.is_alphanumeric() || allow_chars.contains(char))
}

pub(super) fn parameter_validator(keys: &[&str], values: &[&str]) -> Result<(), String> {
    if !keys.iter().all(|key| validate_alphanumeric_name(key, "_")) {
        return Err("The input values as keys is include invalid symbols. keys allow only under bar as symbol.".to_string());
    }

    if keys.len() != values.len() {
        return Err("Length of keys and values (even the conditions) should be match. Please check the input.".to_string());
    }

    Ok(())
}

pub(super) fn validate_string<E, G>(str: &str, param_name: &str, error_generator: &G) -> Result<(), E> where G: ErrorGenerator<E> {
    if !validate_alphanumeric_name(str, "_") {
        let error_message = format!("'{}' has invalid characters. '{}' allows alphabets, numbers and under bar only.", str, param_name);
        return Err(error_generator.generate_error(error_message));
    } else {
        Ok(())
    }
}