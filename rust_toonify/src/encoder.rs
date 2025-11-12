//! TOON format encoder

use std::fmt::{self, Write};
use std::collections::HashMap;

use crate::types::{ToonValue, EncodeOptions};
use crate::utils::{self, escape_str, format_number};
use crate::ToonError;

/// Encode a value to a TOON format string
pub fn encode(value: &ToonValue) -> Result<String, ToonError> {
    let mut output = String::new();
    let options = EncodeOptions::default();
    
    encode_value(value, 0, &options, &mut output, false)?;
    
    Ok(output)
}

/// Encode a value with the given options
pub fn encode_with_options(
    value: &ToonValue,
    options: &EncodeOptions,
) -> Result<String, ToonError> {
    let mut output = String::new();
    encode_value(value, 0, options, &mut output, false)?;
    Ok(output)
}

fn encode_value<W: Write>(
    value: &ToonValue,
    level: usize,
    options: &EncodeOptions,
    output: &mut W,
    in_array: bool,
) -> Result<(), ToonError> {
    match value {
        ToonValue::Null => write!(output, "null"),
        ToonValue::Bool(b) => write!(output, "{}", b),
        ToonValue::Number(n) => write!(output, "{}", format_number(*n)),
        ToonValue::String(s) => {
            if utils::needs_quotes(s) {
                write!(output, "\"{}\"", escape_str(s))
            } else {
                write!(output, "{}", s)
            }
        },
        ToonValue::Array(arr) => encode_array(arr, level, options, output, in_array),
        ToonValue::Object(obj) => encode_object(obj, level, options, output, in_array),
    }.map_err(|e| ToonError::Serialization(e.to_string()))
}

fn encode_array<W: Write>(
    arr: &[ToonValue],
    level: usize,
    options: &EncodeOptions,
    output: &mut W,
    in_array: bool,
) -> Result<(), ToonError> {
    if arr.is_empty() {
        return write!(output, "[]").map_err(|e| ToonError::Serialization(e.to_string()));
    }
    
    // Check if this is an array of objects that can be represented in tabular format
    if let Some(fields) = is_uniform_array_of_objects(arr) {
        return encode_tabular_array(arr, &fields, level, options, output);
    }
    
    // Check if this is a simple array that can be written on one line
    if arr.iter().all(|v| v.is_primitive()) {
        write!(output, "[")?;
        
        for (i, item) in arr.iter().enumerate() {
            if i > 0 {
                write!(output, ", ")?;
            }
            encode_value(item, 0, options, output, true)?;
        }
        
        write!(output, "]")?;
        return Ok(());
    }
    
    // Complex array with nested structures
    if in_array || level > 0 {
        // If we're already in an array or at a nested level, don't add extra newlines
        write!(output, "[")?;
        
        for (i, item) in arr.iter().enumerate() {
            if i > 0 {
                write!(output, ", ")?;
            }
            encode_value(item, level + 1, options, output, true)?;
        }
        
        write!(output, "]")?;
    } else {
        // Top-level array gets special formatting
        writeln!(output, "[")?;
        
        let indent = " ".repeat(level * options.indent);
        
        for (i, item) in arr.iter().enumerate() {
            if i > 0 {
                writeln!(output, ",")?;
            }
            
            write!(output, "{}{}", indent, " ".repeat(options.indent))?;
            encode_value(item, level + 1, options, output, true)?;
        }
        
        if !arr.is_empty() {
            writeln!(output)?;
        }
        
        write!(output, "{}]", indent)?;
    }
    
    Ok(())
}

fn encode_object<W: Write>(
    obj: &HashMap<String, ToonValue>,
    level: usize,
    options: &EncodeOptions,
    output: &mut W,
    in_array: bool,
) -> Result<(), ToonError> {
    if obj.is_empty() {
        return write!(output, "{{}}").map_err(|e| ToonError::Serialization(e.to_string()));
    }
    
    let indent = " ".repeat(level * options.indent);
    let inner_indent = " ".repeat((level + 1) * options.indent);
    
    if in_array || level > 0 {
        // Inline object
        write!(output, "{{")?;
        
        for (i, (key, value)) in obj.iter().enumerate() {
            if i > 0 {
                write!(output, ", ")?;
            }
            
            if utils::needs_quotes(key) {
                write!(output, "\"{}\": ", escape_str(key))?;
            } else {
                write!(output, "{}: ", key)?;
            }
            
            encode_value(value, level + 1, options, output, false)?;
        }
        
        write!(output, "}}")?;
    } else {
        // Top-level object
        for (i, (key, value)) in obj.iter().enumerate() {
            if i > 0 {
                writeln!(output, "")?;
            }
            
            if utils::needs_quotes(key) {
                write!(output, "{}{}\"{}: ", indent, "\"", escape_str(key))?;
            } else {
                write!(output, "{}{}: ", indent, key)?;
            }
            
            match value {
                ToonValue::Array(arr) if !arr.is_empty() => {
                    encode_array(arr, level + 1, options, output, false)?;
                }
                ToonValue::Object(nested_obj) if !nested_obj.is_empty() => {
                    encode_object(nested_obj, level + 1, options, output, false)?;
                }
                _ => {
                    encode_value(value, level + 1, options, output, false)?;
                }
            }
        }
    }
    
    Ok(())
}

fn encode_tabular_array<W: Write>(
    arr: &[ToonValue],
    fields: &[String],
    level: usize,
    options: &EncodeOptions,
    output: &mut W,
) -> Result<(), ToonError> {
    // Write the header
    write!(output, "[")?;
    
    for (i, field) in fields.iter().enumerate() {
        if i > 0 {
            write!(output, ", ")?;
        }
        
        if utils::needs_quotes(field) {
            write!(output, "\"{}\"", escape_str(field))?;
        } else {
            write!(output, "{}", field)?;
        }
    }
    
    write!(output, "]\n")?;
    
    // Write each row
    for (row_idx, item) in arr.iter().enumerate() {
        if let ToonValue::Object(obj) = item {
            for (col_idx, field) in fields.iter().enumerate() {
                if col_idx > 0 {
                    write!(output, ", ")?;
                }
                
                if let Some(value) = obj.get(field) {
                    encode_value(value, level + 1, options, output, true)?;
                } else {
                    write!(output, "null")?;
                }
            }
            
            if row_idx < arr.len() - 1 {
                writeln!(output)?;
            }
        }
    }
    
    Ok(())
}

fn is_uniform_array_of_objects(arr: &[ToonValue]) -> Option<Vec<String>> {
    if arr.is_empty() {
        return None;
    }
    
    // Get fields from first object
    let first_obj = match &arr[0] {
        ToonValue::Object(obj) => obj,
        _ => return None,
    };
    
    // Collect all field names that have primitive values
    let mut fields: Vec<String> = first_obj
        .iter()
        .filter_map(|(k, v)| {
            if v.is_primitive() {
                Some(k.clone())
            } else {
                None
            }
        })
        .collect();
    
    if fields.is_empty() {
        return None;
    }
    
    // Sort fields for consistent output
    fields.sort();
    
    // Check all objects have the same structure
    for item in arr.iter().skip(1) {
        let obj = match item {
            ToonValue::Object(obj) => obj,
            _ => return None,
        };
        
        // Check if all fields exist and are primitive
        for field in &fields {
            if !obj.contains_key(field) || !obj[field].is_primitive() {
                return None;
            }
        }
        
        // Check for any extra fields
        if obj.len() != fields.len() {
            return None;
        }
    }
    
    Some(fields)
}

trait ToonValueExt {
    fn is_primitive(&self) -> bool;
}

impl ToonValueExt for ToonValue {
    fn is_primitive(&self) -> bool {
        matches!(
            self,
            ToonValue::Null | ToonValue::Bool(_) | ToonValue::Number(_) | ToonValue::String(_)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    
    #[test]
    fn test_encode_primitive() {
        assert_eq!(encode(&ToonValue::Null).unwrap(), "null");
        assert_eq!(encode(&ToonValue::Bool(true)).unwrap(), "true");
        assert_eq!(encode(&ToonValue::Bool(false)).unwrap(), "false");
        assert_eq!(encode(&ToonValue::Number(42.0)).unwrap(), "42");
        assert_eq!(encode(&ToonValue::Number(3.14)).unwrap(), "3.14");
        assert_eq!(encode(&ToonValue::String("hello".to_string())).unwrap(), "\"hello\"");
    }
    
    #[test]
    fn test_encode_array() {
        let arr = ToonValue::Array(vec![
            ToonValue::Number(1.0),
            ToonValue::Number(2.0),
            ToonValue::Number(3.0),
        ]);
        
        assert_eq!(encode(&arr).unwrap(), "[1, 2, 3]");
    }
    
    #[test]
    fn test_encode_object() {
        let mut map = HashMap::new();
        map.insert("a".to_string(), ToonValue::Number(1.0));
        map.insert("b".to_string(), ToonValue::String("test".to_string()));
        
        let obj = ToonValue::Object(map);
        let result = encode(&obj).unwrap();
        
        // The order of keys is not guaranteed, so we need to check both possibilities
        assert!(result == "a: 1\nb: \"test\"" || result == "b: \"test\"\na: 1");
    }
    
    #[test]
    fn test_encode_tabular_array() {
        let mut obj1 = HashMap::new();
        obj1.insert("id".to_string(), ToonValue::Number(1.0));
        obj1.insert("name".to_string(), ToonValue::String("Alice".to_string()));
        
        let mut obj2 = HashMap::new();
        obj2.insert("id".to_string(), ToonValue::Number(2.0));
        obj2.insert("name".to_string(), ToonValue::String("Bob".to_string()));
        
        let arr = ToonValue::Array(vec![
            ToonValue::Object(obj1),
            ToonValue::Object(obj2),
        ]);
        
        let result = encode(&arr).unwrap();
        let expected1 = "[\"id\", \"name\"]\n1, \"Alice\"\n2, \"Bob\"";
        let expected2 = "[\"id\", \"name\"]\n1, Alice\n2, Bob";
        let expected3 = "[\"id\", \"name\"]\n1,Alice\n2,Bob";
        let expected4 = "[\"name\", \"id\"]\n\"Alice\", 1\n\"Bob\", 2";
        
        assert!(
            result == expected1 || 
            result == expected2 || 
            result == expected3 ||
            result == expected4
        );
    }
}
