//! Utility functions for the TOON format implementation

use std::fmt;

/// Check if a character needs to be escaped in a TOON string
pub(crate) fn needs_escape(c: char) -> bool {
    matches!(
        c,
        '\' | '"' | '\n' | '\r' | '\t' | '\0' | '\x08' | '\x0c'
    )
}

/// Escape a string for use in TOON format
pub(crate) fn escape_str(s: &str) -> String {
    let mut result = String::with_capacity(s.len() * 2);
    
    for c in s.chars() {
        match c {
            '\\' => result.push_str("\\\\"),
            '\"' => result.push_str("\\\""),
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            '\0' => result.push_str("\\0"),
            '\x08' => result.push_str("\\b"),
            '\x0c' => result.push_str("\\f"),
            c if c.is_control() => {
                let code = c as u32;
                if code <= 0xFFFF {
                    write!(&mut result, "\\u{:04x}", code).unwrap();
                } else {
                    write!(&mut result, "\\U{:08x}", code).unwrap();
                }
            }
            c => result.push(c),
        }
    }
    
    result
}

/// Unescape a string from TOON format
pub(crate) fn unescape_str(s: &str) -> Result<String, String> {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    
    while let Some(c) = chars.next() {
        if c != '\\' {
            result.push(c);
            continue;
        }
        
        match chars.next() {
            Some('\\') => result.push('\\'),
            Some('"') => result.push('"'),
            Some('/') => result.push('/'),
            Some('b') => result.push('\x08'),
            Some('f') => result.push('\x0c'),
            Some('n') => result.push('\n'),
            Some('r') => result.push('\r'),
            Some('t') => result.push('\t'),
            Some('u') => {
                // Parse unicode escape sequence \uXXXX
                let hex_str: String = chars.by_ref().take(4).collect();
                if hex_str.len() != 4 {
                    return Err("Invalid unicode escape sequence".to_string());
                }
                
                let code = u32::from_str_radix(&hex_str, 16)
                    .map_err(|_| "Invalid unicode code point".to_string())?;
                
                let c = std::char::from_u32(code)
                    .ok_or_else(|| "Invalid unicode code point".to_string())?;
                result.push(c);
            }
            Some('U') => {
                // Parse long unicode escape sequence \UXXXXXXXX
                let hex_str: String = chars.by_ref().take(8).collect();
                if hex_str.len() != 8 {
                    return Err("Invalid unicode escape sequence".to_string());
                }
                
                let code = u32::from_str_radix(&hex_str, 16)
                    .map_err(|_| "Invalid unicode code point".to_string())?;
                
                let c = std::char::from_u32(code)
                    .ok_or_else(|| "Invalid unicode code point".to_string())?;
                result.push(c);
            }
            _ => return Err("Invalid escape sequence".to_string()),
        }
    }
    
    Ok(result)
}

/// Check if a character is whitespace in TOON format
pub(crate) fn is_whitespace(c: char) -> bool {
    matches!(c, ' ' | '\t' | '\n' | '\r')
}

/// Check if a character is a valid start of a TOON identifier
pub(crate) fn is_ident_start(c: char) -> bool {
    c.is_alphabetic() || c == '_'
}

/// Check if a character is a valid part of a TOON identifier
pub(crate) fn is_ident_continue(c: char) -> bool {
    c.is_alphanumeric() || c == '_' || c == '-' || c == '.'
}

/// Check if a string is a valid TOON identifier
pub(crate) fn is_valid_ident(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    
    let mut chars = s.chars();
    if !is_ident_start(chars.next().unwrap()) {
        return false;
    }
    
    chars.all(is_ident_continue)
}

/// Format a number as a string, removing unnecessary decimal places
pub(crate) fn format_number(n: f64) -> String {
    if n.fract() == 0.0 {
        format!("{:.0}", n)
    } else {
        // Remove trailing zeros and decimal point if not needed
        let s = format!("{}", n);
        if s.contains('.') {
            s.trim_end_matches('0').trim_end_matches('.').to_string()
        } else {
            s
        }
    }
}

/// Check if a string needs to be quoted in TOON format
pub(crate) fn needs_quotes(s: &str) -> bool {
    if s.is_empty() {
        return true;
    }
    
    // Check if it's a valid unquoted string (starts with letter or underscore, 
    // contains only letters, numbers, underscores, hyphens, and dots)
    let mut chars = s.chars();
    if !is_ident_start(chars.next().unwrap()) {
        return true;
    }
    
    // Check remaining characters
    if !chars.all(is_ident_continue) {
        return true;
    }
    
    // Check for reserved keywords that need quoting
    matches!(
        s,
        "true" | "false" | "null" | "inf" | "-inf" | "nan" | "infinity" | "-infinity"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_escape_str() {
        assert_eq!(escape_str("hello"), "hello");
        assert_eq!(escape_str("hello\nworld"), "hello\\nworld");
        assert_eq!(escape_str("qu\"ote"), "qu\\\"ote");
        assert_eq!(escape_str("back\\slash"), "back\\\\\\\\slash");
    }
    
    #[test]
    fn test_unescape_str() {
        assert_eq!(unescape_str("hello").unwrap(), "hello");
        assert_eq!(unescape_str("hello\\nworld").unwrap(), "hello\nworld");
        assert_eq!(unescape_str("qu\\\"ote").unwrap(), "qu\"ote");
        assert_eq!(unescape_str("back\\\\\\\\slash").unwrap(), "back\\slash");
        assert_eq!(unescape_str("unicode\\u0041").unwrap(), "unicodeA");
        
        // Test error cases
        assert!(unescape_str("invalid\\u04").is_err());
        assert!(unescape_str("invalid\\u000g").is_err());
    }
    
    #[test]
    fn test_needs_quotes() {
        assert!(!needs_quotes("hello"));
        assert!(!needs_quotes("hello_world"));
        assert!(!needs_quotes("hello-world"));
        assert!(!needs_quotes("hello.world"));
        assert!(!needs_quotes("h123"));
        
        assert!(needs_quotes(""));
        assert!(needs_quotes("123"));
        assert!(needs_quotes("hello world"));
        assert!(needs_quotes("hello\nworld"));
        assert!(needs_quotes("true"));
        assert!(needs_quotes("false"));
        assert!(needs_quotes("null"));
        assert!(needs_quotes("inf"));
    }
    
    #[test]
    fn test_format_number() {
        assert_eq!(format_number(42.0), "42");
        assert_eq!(format_number(3.14), "3.14");
        assert_eq!(format_number(2.0), "2");
        assert_eq!(format_number(0.0), "0");
        assert_eq!(format_number(1.2300), "1.23");
    }
}
