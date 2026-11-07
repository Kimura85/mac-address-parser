//! # MAC Address Parser
//!
//! Parses MAC addresses in colon, hyphen, and dot notations into a fixed
//! six-byte representation.
//!
//! The parser accepts the three common notations:
//!
//! - Colon-separated: `aa:bb:cc:dd:ee:ff`
//! - Hyphen-separated: `aa-bb-cc-dd-ee-ff`
//! - Dot-separated (Cisco-style): `aabb.ccdd.eeff`
//!
//! All hex digits are case-insensitive. Separators must be consistent within
//! a single address; mixing separators is rejected.

/// A MAC address stored as six bytes in network order.
///
/// We store the bytes rather than the original string so that equality and
/// formatting are canonical: two addresses that differ only in notation or
/// letter case compare equal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MacAddress([u8; 6]);

impl MacAddress {
    /// Returns the six bytes of the address in network order.
    pub fn octets(&self) -> [u8; 6] {
        self.0
    }

    /// Parses a MAC address string.
    ///
    /// Accepts colon (`aa:bb:cc:dd:ee:ff`), hyphen (`aa-bb-cc-dd-ee-ff`),
    /// and dot (`aabb.ccdd.eeff`) notations. Hex digits are case-insensitive.
    /// Separators must be consistent within a single address.
    ///
    /// Returns `Err` with a human-readable message on malformed input.
    pub fn parse(s: &str) -> Result<MacAddress, String> {
        if s.is_empty() {
            return Err("empty MAC address".to_string());
        }

        // Detect the separator by looking at the first non-hex character.
        // Using the first separator (rather than the last) keeps the decision
        // deterministic even for strings like "aa:bb-cc", which we reject.
        let mut sep: Option<char> = None;
        for c in s.chars() {
            if c == ':' || c == '-' || c == '.' {
                sep = Some(c);
                break;
            }
        }

        let sep = match sep {
            Some(c) => c,
            None => {
                // No separator at all: require exactly 12 hex digits.
                return parse_bare(s);
            }
        };

        // Reject mixed separators up front so the split logic below can
        // assume a single delimiter.
        for c in s.chars() {
            if (c == ':' || c == '-' || c == '.') && c != sep {
                return Err(format!(
                    "inconsistent separator: found '{}' and '{}'",
                    sep, c
                ));
            }
        }

        match sep {
            ':' | '-' => parse_pairs(s, sep),
            '.' => parse_dotted(s),
            _ => unreachable!(),
        }
    }
}

/// Parses a bare 12-hex-digit string with no separators.
fn parse_bare(s: &str) -> Result<MacAddress, String> {
    if s.len() != 12 {
        return Err(format!(
            "expected 12 hex digits, got {}",
            s.len()
        ));
    }
    let mut bytes = [0u8; 6];
    let chars: Vec<char> = s.chars().collect();
    for i in 0..6 {
        let hi = hex_val(chars[2 * i])?;
        let lo = hex_val(chars[2 * i + 1])?;
        bytes[i] = (hi << 4) | lo;
    }
    Ok(MacAddress(bytes))
}

/// Parses colon- or hyphen-separated pairs: `aa:bb:cc:dd:ee:ff`.
fn parse_pairs(s: &str, sep: char) -> Result<MacAddress, String> {
    let parts: Vec<&str> = s.split(sep).collect();
    if parts.len() != 6 {
        return Err(format!(
            "expected 6 groups separated by '{}', got {}",
            sep,
            parts.len()
        ));
    }
    let mut bytes = [0u8; 6];
    for (i, part) in parts.iter().enumerate() {
        if part.len() != 2 {
            return Err(format!(
                "group {} must be exactly 2 hex digits, got {:?}",
                i, part
            ));
        }
        let mut it = part.chars();
        let hi = hex_val(it.next().unwrap())?;
        let lo = hex_val(it.next().unwrap())?;
        bytes[i] = (hi << 4) | lo;
    }
    Ok(MacAddress(bytes))
}

/// Parses dot-separated groups of four hex digits: `aabb.ccdd.eeff`.
fn parse_dotted(s: &str) -> Result<MacAddress, String> {
    let parts: Vec<&str> = s.split('.').collect();
    if parts.len() != 3 {
        return Err(format!(
            "expected 3 groups separated by '.', got {}",
            parts.len()
        ));
    }
    let mut bytes = [0u8; 6];
    for (i, part) in parts.iter().enumerate() {
        if part.len() != 4 {
            return Err(format!(
                "group {} must be exactly 4 hex digits, got {:?}",
                i, part
            ));
        }
        let mut it = part.chars();
        let b0 = hex_val(it.next().unwrap())?;
        let b1 = hex_val(it.next().unwrap())?;
        let b2 = hex_val(it.next().unwrap())?;
        let b3 = hex_val(it.next().unwrap())?;
        bytes[2 * i] = (b0 << 4) | b1;
        bytes[2 * i + 1] = (b2 << 4) | b3;
    }
    Ok(MacAddress(bytes))
}

/// Converts a single hex digit character to its numeric value.
fn hex_val(c: char) -> Result<u8, String> {
    match c {
        '0'..='9' => Ok(c as u8 - b'0'),
        'a'..='f' => Ok(c as u8 - b'a' + 10),
        'A'..='F' => Ok(c as u8 - b'A' + 10),
        _ => Err(format!("invalid hex digit: {:?}", c)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_colon_notation() {
        let m = MacAddress::parse("aa:bb:cc:dd:ee:ff").unwrap();
        assert_eq!(m.octets(), [0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff]);
    }

    #[test]
    fn parses_hyphen_notation() {
        let m = MacAddress::parse("AA-BB-CC-DD-EE-FF").unwrap();
        assert_eq!(m.octets(), [0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff]);
    }

    #[test]
    fn parses_dot_notation() {
        let m = MacAddress::parse("aabb.ccdd.eeff").unwrap();
        assert_eq!(m.octets(), [0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff]);
    }

    #[test]
    fn parses_bare_hex() {
        let m = MacAddress::parse("aabbccddeeff").unwrap();
        assert_eq!(m.octets(), [0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff]);
    }

    #[test]
    fn all_zero() {
        let m = MacAddress::parse("00:00:00:00:00:00").unwrap();
        assert_eq!(m.octets(), [0, 0, 0, 0, 0, 0]);
    }

    #[test]
    fn all_ff() {
        let m = MacAddress::parse("ff:ff:ff:ff:ff:ff").unwrap();
        assert_eq!(m.octets(), [0xff; 6]);
    }

    #[test]
    fn mixed_case_is_accepted() {
        let m = MacAddress::parse("aA:bB:cC:dD:eE:fF").unwrap();
        assert_eq!(m.octets(), [0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff]);
    }

    #[test]
    fn equivalent_notations_compare_equal() {
        let a = MacAddress::parse("aa:bb:cc:dd:ee:ff").unwrap();
        let b = MacAddress::parse("AA-BB-CC-DD-EE-FF").unwrap();
        let c = MacAddress::parse("aabb.ccdd.eeff").unwrap();
        assert_eq!(a, b);
        assert_eq!(b, c);
    }

    #[test]
    fn rejects_empty() {
        assert!(MacAddress::parse("").is_err());
    }

    #[test]
    fn rejects_wrong_group_count_colon() {
        let err = MacAddress::parse("aa:bb:cc:dd:ee").unwrap_err();
        assert!(err.contains("6 groups"), "unexpected error: {}", err);
    }

    #[test]
    fn rejects_wrong_group_count_dot() {
        let err = MacAddress::parse("aabb.ccdd").unwrap_err();
        assert!(err.contains("3 groups"), "unexpected error: {}", err);
    }

    #[test]
    fn rejects_short_group() {
        assert!(MacAddress::parse("a:b:c:d:e:f").is_err());
    }

    #[test]
    fn rejects_long_group() {
        assert!(MacAddress::parse("aaa:bbb:ccc:ddd:eee:fff").is_err());
    }

    #[test]
    fn rejects_non_hex_digit() {
        let err = MacAddress::parse("gg:bb:cc:dd:ee:ff").unwrap_err();
        assert!(err.contains("invalid hex digit"), "unexpected error: {}", err);
    }

    #[test]
    fn rejects_mixed_separators() {
        let err = MacAddress::parse("aa:bb-cc:dd:ee:ff").unwrap_err();
        assert!(err.contains("inconsistent separator"), "unexpected error: {}", err);
    }

    #[test]
    fn rejects_bare_wrong_length() {
        assert!(MacAddress::parse("aabbccddee").is_err());
        assert!(MacAddress::parse("aabbccddeeff00").is_err());
    }

    #[test]
    fn rejects_trailing_separator() {
        assert!(MacAddress::parse("aa:bb:cc:dd:ee:ff:").is_err());
    }

    #[test]
    fn rejects_whitespace() {
        assert!(MacAddress::parse(" aa:bb:cc:dd:ee:ff").is_err());
        assert!(MacAddress::parse("aa:bb:cc:dd:ee:ff ").is_err());
    }
}
