use subtle::ConstantTimeEq;

// Compares two secrets or tokens without stopping at the first different byte (D35, D39)
pub fn secrets_match(left: &str, right: &str) -> bool {
    bool::from(left.as_bytes().ct_eq(right.as_bytes()))
}

#[cfg(test)]
mod tests {
    use super::*;

    // Equal values match
    #[test]
    fn equal_values_match() {
        assert!(secrets_match("test-secret", "test-secret"));
    }

    // Two empty values match
    #[test]
    fn empty_values_match() {
        assert!(secrets_match("", ""));
    }

    // Values that differ in one character do not match
    #[test]
    fn different_values_do_not_match() {
        assert!(!secrets_match("test-secret", "test-secreT"));
    }

    // Values of different length do not match, even when one is a prefix of the other
    #[test]
    fn different_length_values_do_not_match() {
        assert!(!secrets_match("test-secret", "test-secret-longer"));
        assert!(!secrets_match("test-secret", ""));
    }
}
