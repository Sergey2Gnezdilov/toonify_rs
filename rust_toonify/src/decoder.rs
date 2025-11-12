//! TOON format decoder

use std::collections::HashMap;
use std::str::Chars;

use crate::types::ToonValue;
use crate::utils::{self, unescape_str};
use crate::ToonError;

/// Parse a TOON string into a `ToonValue`
pub fn decode(input: &str) -> Result<ToonValue, ToonError> {
    let mut parser = Parser::new(input);
    parser.parse()
}

/// Parser state for the TOON format
struct Parser<'a> {
    chars: Chars<'a>,
    current: Option<char>,
    line: usize,
    col: usize,
}

impl<'a> Parser<'a> {
    /// Create a new parser for the given input string
    fn new(input: &'a str) -> Self {
        let mut chars = input.chars();
        let current = chars.next();
        
        Self {
            chars,
            current,
            line: 1,
            col: 1,
        }
    }
    
    /// Advance to the next character
    fn next(&mut self) -> Option<char> {
        self.current = self.chars.next();
        
        if let Some(c) = self.current {
            if c == '\n' {
                self.line += 1;
                self.col = 1;
            } else {
                self.col += 1;
            }
        }
        
        self.current
    }
    
    /// Skip whitespace characters
    fn skip_whitespace(&mut self) {
        while let Some(c) = self.current {
            if !c.is_whitespace() {
                break;
            }
            self.next();
        }
    }
    
    /// Parse the input string into a `ToonValue`
    fn parse(&mut self) -> Result<ToonValue, ToonError> {
        self.skip_whitespace();
        
        match self.current {
            Some('{') => self.parse_object(),
            Some('[') => self.parse_array(),
            Some('"') => self.parse_string(),
            Some('t') => self.parse_keyword("true", ToonValue::Bool(true)),
            Some('f') => self.parse_keyword("false", ToonValue::Bool(false)),
            Some('n') => self.parse_keyword("null", ToonValue::Null),
            Some(c) if c.is_ascii_digit() || c == '-' => self.parse_number(),
            Some(c) if utils::is_ident_start(c) => self.parse_identifier(),
            Some(c) => Err(ToonError::InvalidFormat(format!(
                "Unexpected character '{}' at line {}, column {}",
                c, self.line, self.col
            ))),
            None => Err(ToonError::InvalidFormat("Unexpected end of input".to_string())),
        }
    }
    
    /// Parse a JSON object
    fn parse_object(&mut self) -> Result<ToonValue, ToonError> {
        assert_eq!(self.current, Some('{'));
        self.next(); // Skip '{'
        
        let mut obj = HashMap::new();
        
        // Handle empty object
        self.skip_whitespace();
        if self.current == Some('}') {
            self.next();
            return Ok(ToonValue::Object(obj));
        }
        
        loop {
            // Parse key
            self.skip_whitespace();
            let key = match self.current {
                Some('"') => self.parse_string()?,
                Some(c) if utils::is_ident_start(c) => self.parse_identifier()?,
                Some(ch) => {
                    return Err(ToonError::InvalidFormat(format!(
                        "Expected string or identifier at line {}, column {}, found '{}'",
                        self.line, self.col, ch
                    )));
                }
                None => {
                    return Err(ToonError::InvalidFormat(
                        "Unexpected end of input while parsing object".to_string(),
                    ));
                }
            };
            
            let key = match key {
                ToonValue::String(s) => s,
                _ => unreachable!("parse_string and parse_identifier return String"),
            };
            
            // Parse ':'
            self.skip_whitespace();
            if self.current != Some(':') {
                return Err(ToonError::InvalidFormat(format!(
                    "Expected ':' after key at line {}, column {}",
                    self.line, self.col
                )));
            }
            self.next();
            
            // Parse value
            self.skip_whitespace();
            let value = self.parse()?;
            
            // Insert into object
            obj.insert(key, value);
            
            // Parse ',' or '}'
            self.skip_whitespace();
            match self.current {
                Some(',') => {
                    self.next();
                    continue;
                }
                Some('}') => {
                    self.next();
                    break;
                }
                _ => {
                    return Err(ToonError::InvalidFormat(format!(
                        "Expected ',' or '}}' at line {}, column {}",
                        self.line, self.col
                    )));
                }
            }
        }
        
        Ok(ToonValue::Object(obj))
    }
    
    /// Parse a JSON array
    fn parse_array(&mut self) -> Result<ToonValue, ToonError> {
        assert_eq!(self.current, Some('['));
        self.next(); // Skip '['
        
        let mut arr = Vec::new();
        
        // Handle empty array
        self.skip_whitespace();
        if self.current == Some(']') {
            self.next();
            return Ok(ToonValue::Array(arr));
        }
        
        loop {
            // Parse value
            self.skip_whitespace();
            let value = self.parse()?;
            arr.push(value);
            
            // Parse ',' or ']'
            self.skip_whitespace();
            match self.current {
                Some(',') => {
                    self.next();
                    continue;
                }
                Some(']') => {
                    self.next();
                    break;
                }
                _ => {
                    return Err(ToonError::InvalidFormat(format!(
                        "Expected ',' or ']' at line {}, column {}",
                        self.line, self.col
                    )));
                }
            }
        }
        
        Ok(ToonValue::Array(arr))
    }
    
    /// Parse a string value
    fn parse_string(&mut self) -> Result<ToonValue, ToonError> {
        assert_eq!(self.current, Some('"'));
        self.next(); // Skip opening '"'
        
        let mut s = String::new();
        
        while let Some(c) = self.current {
            match c {
                '\"' => {
                    self.next();
                    break;
                }
                '\\' => {
                    self.next(); // Skip '\\'
                    let escaped = match self.current {
                        Some('"') => '"',
                        Some('\\') => '\\',
                        Some('/') => '/',
                        Some('b') => '\x08',
                        Some('f') => '\x0c',
                        Some('n') => '\n',
                        Some('r') => '\r',
                        Some('t') => '\t',
                        Some('u') => {
                            // Parse unicode escape sequence \uXXXX
                            self.next(); // Skip 'u'
                            let hex = self.take_chars(4);
                            if hex.len() != 4 {
                                return Err(ToonError::InvalidFormat(
                                    "Invalid unicode escape sequence".to_string(),
                                ));
                            }
                            
                            let code = u32::from_str_radix(&hex, 16).map_err(|_| {
                                ToonError::InvalidFormat("Invalid unicode code point".to_string())
                            })?;
                            
                            std::char::from_u32(code).ok_or_else(|| {
                                ToonError::InvalidFormat("Invalid unicode code point".to_string())
                            })?
                        }
                        _ => {
                            return Err(ToonError::InvalidFormat(format!(
                                "Invalid escape sequence at line {}, column {}",
                                self.line, self.col
                            )));
                        }
                    };
                    
                    s.push(escaped);
                    self.next();
                }
                _ => {
                    s.push(c);
                    self.next();
                }
            }
        }
        
        // Unescape the string
        let unescaped = unescape_str(&s).map_err(|e| ToonError::Deserialization(e))?;
        
        Ok(ToonValue::String(unescaped))
    }
    
    /// Parse a number value
    fn parse_number(&mut self) -> Result<ToonValue, ToonError> {
        let mut num_str = String::new();
        let mut has_decimal = false;
        let mut has_exponent = false;
        
        // Handle sign
        if self.current == Some('-') {
            num_str.push('-' as u8 as char);
            self.next();
        }
        
        // Parse integer part
        while let Some(c) = self.current {
            if c.is_ascii_digit() {
                num_str.push(c);
                self.next();
            } else {
                break;
            }
        }
        
        // Parse fractional part
        if self.current == Some('.') {
            has_decimal = true;
            num_str.push('.' as u8 as char);
            self.next();
            
            let mut has_digits = false;
            while let Some(c) = self.current {
                if c.is_ascii_digit() {
                    has_digits = true;
                    num_str.push(c);
                    self.next();
                } else {
                    break;
                }
            }
            
            if !has_digits {
                return Err(ToonError::InvalidFormat(
                    "Expected digit after decimal point".to_string(),
                ));
            }
        }
        
        // Parse exponent
        if self.current == Some('e') || self.current == Some('E') {
            has_exponent = true;
            num_str.push('e' as u8 as char);
            self.next();
            
            if self.current == Some('+') || self.current == Some('-') {
                num_str.push(self.current.unwrap());
                self.next();
            }
            
            let mut has_digits = false;
            while let Some(c) = self.current {
                if c.is_ascii_digit() {
                    has_digits = true;
                    num_str.push(c);
                    self.next();
                } else {
                    break;
                }
            }
            
            if !has_digits {
                return Err(ToonError::InvalidFormat(
                    "Expected digit in exponent".to_string(),
                ));
            }
        }
        
        // Parse the number
        if has_decimal || has_exponent {
            num_str.parse::<f64>()
                .map(ToonValue::Number)
                .map_err(|e| ToonError::Deserialization(e.to_string()))
        } else {
            num_str.parse::<i64>()
                .map(|n| ToonValue::Number(n as f64))
                .or_else(|_| {
                    num_str.parse::<f64>()
                        .map(ToonValue::Number)
                        .map_err(|e| ToonError::Deserialization(e.to_string()))
                })
        }
    }
    
    /// Parse a keyword (true, false, null)
    fn parse_keyword(
        &mut self,
        keyword: &str,
        value: ToonValue,
    ) -> Result<ToonValue, ToonError> {
        let s = self.take_chars(keyword.len());
        
        if s == keyword {
            Ok(value)
        } else {
            Err(ToonError::InvalidFormat(format!(
                "Unexpected token '{}', expected '{}' at line {}, column {}",
                s, keyword, self.line, self.col
            )))
        }
    }
    
    /// Parse an unquoted identifier
    fn parse_identifier(&mut self) -> Result<ToonValue, ToonError> {
        let mut ident = String::new();
        
        // First character must be a letter or underscore
        if let Some(c) = self.current {
            if utils::is_ident_start(c) {
                ident.push(c);
                self.next();
            } else {
                return Err(ToonError::InvalidFormat(format!(
                    "Expected identifier start at line {}, column {}",
                    self.line, self.col
                )));
            }
        }
        
        // Subsequent characters can be letters, digits, underscores, hyphens, or dots
        while let Some(c) = self.current {
            if utils::is_ident_continue(c) {
                ident.push(c);
                self.next();
            } else {
                break;
            }
        }
        
        // Check for reserved keywords
        match ident.as_str() {
            "true" => Ok(ToonValue::Bool(true)),
            "false" => Ok(ToonValue::Bool(false)),
            "null" => Ok(ToonValue::Null),
            _ => Ok(ToonValue::String(ident)),
        }
    }
    
    /// Take the next `count` characters as a `String`
    fn take_chars(&mut self, count: usize) -> String {
        let mut buf = String::with_capacity(count);
        for _ in 0..count {
            if let Some(c) = self.current {
                buf.push(c);
                self.next();
            } else {
                break;
            }
        }
        buf
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    
    #[test]
    fn test_parse_primitive() {
        assert_eq!(decode("null").unwrap(), ToonValue::Null);
        assert_eq!(decode("true").unwrap(), ToonValue::Bool(true));
        assert_eq!(decode("false").unwrap(), ToonValue::Bool(false));
        assert_eq!(decode("42").unwrap(), ToonValue::Number(42.0));
        assert_eq!(decode("3.14").unwrap(), ToonValue::Number(3.14));
        assert_eq!(
            decode("\"hello\"").unwrap(),
            ToonValue::String("hello".to_string())
        );
    }
    
    #[test]
    fn test_parse_array() {
        assert_eq!(decode("[]").unwrap(), ToonValue::Array(vec![]));
        
        assert_eq!(
            decode("[1, 2, 3]").unwrap(),
            ToonValue::Array(vec![
                ToonValue::Number(1.0),
                ToonValue::Number(2.0),
                ToonValue::Number(3.0),
            ])
        );
        
        assert_eq!(
            decode("[\"a\", \"b\", \"c\"]").unwrap(),
            ToonValue::Array(vec![
                ToonValue::String("a".to_string()),
                ToonValue::String("b".to_string()),
                ToonValue::String("c".to_string()),
            ])
        );
    }
    
    #[test]
    fn test_parse_object() {
        assert_eq!(decode("{}").unwrap(), ToonValue::Object(HashMap::new()));
        
        let mut expected = HashMap::new();
        expected.insert("a".to_string(), ToonValue::Number(1.0));
        expected.insert("b".to_string(), ToonValue::Number(2.0));
        
        let result = decode("{\"a\": 1, \"b\": 2}").unwrap();
        assert_eq!(result, ToonValue::Object(expected.clone()));
        
        // Test with unquoted keys
        let result = decode("{a: 1, b: 2}").unwrap();
        assert_eq!(result, ToonValue::Object(expected));
    }
    
    #[test]
    fn test_parse_nested() {
        let input = r#"{
            "name": "John",
            "age": 30,
            "address": {
                "street": "123 Main St",
                "city": "Anytown"
            },
            "hobbies": ["reading", "swimming", "coding"]
        }"#;
        
        let result = decode(input);
        assert!(result.is_ok());
        
        if let Ok(ToonValue::Object(obj)) = result {
            assert_eq!(obj.get("name"), Some(&ToonValue::String("John".to_string())));
            assert_eq!(obj.get("age"), Some(&ToonValue::Number(30.0)));
            
            if let Some(ToonValue::Object(address)) = obj.get("address") {
                assert_eq!(
                    address.get("street"),
                    Some(&ToonValue::String("123 Main St".to_string()))
                );
                assert_eq!(
                    address.get("city"),
                    Some(&ToonValue::String("Anytown".to_string()))
                );
            } else {
                panic!("Expected address to be an object");
            }
            
            if let Some(ToonValue::Array(hobbies)) = obj.get("hobbies") {
                assert_eq!(hobbies.len(), 3);
                assert_eq!(hobbies[0], ToonValue::String("reading".to_string()));
                assert_eq!(hobbies[1], ToonValue::String("swimming".to_string()));
                assert_eq!(hobbies[2], ToonValue::String("coding".to_string()));
            } else {
                panic!("Expected hobbies to be an array");
            }
        } else {
            panic!("Expected root to be an object");
        }
    }
}
