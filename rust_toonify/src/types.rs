//! Core data types for the TOON format

use std::collections::HashMap;
use std::fmt;

/// Represents a value in the TOON format
#[derive(Debug, Clone, PartialEq)]
pub enum ToonValue {
    /// Represents a null value
    Null,
    /// Represents a boolean value
    Bool(bool),
    /// Represents a numeric value (f64 can represent all JSON numbers)
    Number(f64),
    /// Represents a string value
    String(String),
    /// Represents an array of values
    Array(Vec<ToonValue>),
    /// Represents an object with string keys and ToonValue values
    Object(HashMap<String, ToonValue>),
}

impl ToonValue {
    /// Check if the value is null
    pub fn is_null(&self) -> bool {
        matches!(self, ToonValue::Null)
    }

    /// Get the value as a boolean if it is one
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            ToonValue::Bool(b) => Some(*b),
            _ => None,
        }
    }

    /// Get the value as a number if it is one
    pub fn as_number(&self) -> Option<f64> {
        match self {
            ToonValue::Number(n) => Some(*n),
            _ => None,
        }
    }

    /// Get the value as a string slice if it is one
    pub fn as_str(&self) -> Option<&str> {
        match self {
            ToonValue::String(s) => Some(s.as_str()),
            _ => None,
        }
    }

    /// Get the value as a mutable string slice if it is one
    pub fn as_str_mut(&mut self) -> Option<&mut String> {
        match self {
            ToonValue::String(s) => Some(s),
            _ => None,
        }
    }

    /// Get the value as a slice of ToonValues if it is an array
    pub fn as_array(&self) -> Option<&[ToonValue]> {
        match self {
            ToonValue::Array(arr) => Some(arr.as_slice()),
            _ => None,
        }
    }

    /// Get the value as a mutable slice of ToonValues if it is an array
    pub fn as_array_mut(&mut self) -> Option<&mut Vec<ToonValue>> {
        match self {
            ToonValue::Array(arr) => Some(arr),
            _ => None,
        }
    }

    /// Get the value as a reference to the inner HashMap if it is an object
    pub fn as_object(&self) -> Option<&HashMap<String, ToonValue>> {
        match self {
            ToonValue::Object(map) => Some(map),
            _ => None,
        }
    }

    /// Get the value as a mutable reference to the inner HashMap if it is an object
    pub fn as_object_mut(&mut self) -> Option<&mut HashMap<String, ToonValue>> {
        match self {
            ToonValue::Object(map) => Some(map),
            _ => None,
        }
    }
}

impl fmt::Display for ToonValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ToonValue::Null => write!(f, "null"),
            ToonValue::Bool(b) => write!(f, "{}", b),
            ToonValue::Number(n) => {
                // Format integers without decimal part for better readability
                if n.fract() == 0.0 {
                    write!(f, "{:.0}", n)
                } else {
                    write!(f, "{}", n)
                }
            }
            ToonValue::String(s) => write!(f, "\"{}\"", s.escape_default()),
            ToonValue::Array(arr) => {
                write!(f, "[")?;
                for (i, item) in arr.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", item)?;
                }
                write!(f, "]")
            }
            ToonValue::Object(obj) => {
                write!(f, "{{")?;
                for (i, (k, v)) in obj.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "\"{}\": {}", k.escape_default(), v)?;
                }
                write!(f, "}}")
            }
        }
    }
}

/// Options for encoding ToonValue to a string
#[derive(Debug, Clone, Copy)]
pub struct EncodeOptions {
    /// Whether to pretty-print the output
    pub pretty: bool,
    /// Number of spaces to use for indentation (if pretty-printing)
    pub indent: usize,
    /// Whether to escape non-ASCII characters
    pub escape_non_ascii: bool,
}

impl Default for EncodeOptions {
    fn default() -> Self {
        Self {
            pretty: false,
            indent: 2,
            escape_non_ascii: false,
        }
    }
}

impl EncodeOptions {
    /// Create a new EncodeOptions with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Set whether to pretty-print the output
    pub fn pretty(mut self, pretty: bool) -> Self {
        self.pretty = pretty;
        self
    }

    /// Set the number of spaces to use for indentation
    pub fn indent(mut self, indent: usize) -> Self {
        self.indent = indent;
        self
    }

    /// Set whether to escape non-ASCII characters
    pub fn escape_non_ascii(mut self, escape: bool) -> Self {
        self.escape_non_ascii = escape;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_toon_value_display() {
        assert_eq!(ToonValue::Null.to_string(), "null");
        assert_eq!(ToonValue::Bool(true).to_string(), "true");
        assert_eq!(ToonValue::Bool(false).to_string(), "false");
        assert_eq!(ToonValue::Number(42.0).to_string(), "42");
        assert_eq!(ToonValue::Number(3.14).to_string(), "3.14");
        assert_eq!(
            ToonValue::String("hello".to_string()).to_string(),
            "\"hello\""
        );
        assert_eq!(
            ToonValue::String("qu\"ote".to_string()).to_string(),
            "\"qu\\\"ote\""
        );
        
        let array = ToonValue::Array(vec![
            ToonValue::Number(1.0),
            ToonValue::Number(2.0),
            ToonValue::Number(3.0),
        ]);
        assert_eq!(array.to_string(), "[1, 2, 3]");
        
        let mut map = HashMap::new();
        map.insert("a".to_string(), ToonValue::Number(1.0));
        map.insert("b".to_string(), ToonValue::Number(2.0));
        let obj = ToonValue::Object(map);
        
        // The order of keys is not guaranteed, so we need to check both possibilities
        let s = obj.to_string();
        assert!(s == "{\"a\": 1, \"b\": 2}" || s == "{\"b\": 2, \"a\": 1}");
    }

    #[test]
    fn test_as_methods() {
        let null = ToonValue::Null;
        assert!(null.is_null());
        assert_eq!(null.as_bool(), None);
        
        let bool_val = ToonValue::Bool(true);
        assert_eq!(bool_val.as_bool(), Some(true));
        
        let num = ToonValue::Number(42.0);
        assert_eq!(num.as_number(), Some(42.0));
        
        let s = ToonValue::String("test".to_string());
        assert_eq!(s.as_str(), Some("test"));
        
        let arr = ToonValue::Array(vec![ToonValue::Number(1.0)]);
        assert_eq!(arr.as_array().map(|a| a.len()), Some(1));
        
        let mut map = HashMap::new();
        map.insert("key".to_string(), ToonValue::String("value".to_string()));
        let obj = ToonValue::Object(map);
        assert_eq!(obj.as_object().map(|m| m.len()), Some(1));
    }
}
