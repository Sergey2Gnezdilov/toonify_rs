//! # Rust TOON Format Implementation
//! 
//! A high-performance implementation of the TOON format in Rust with Python bindings.

use std::collections::HashMap;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use thiserror::Error;

// Re-export public API
pub mod encoder;
pub mod decoder;
pub mod utils;
pub mod types;

use types::ToonValue;

/// Error type for TOON encoding/decoding operations
#[derive(Error, Debug)]
pub enum ToonError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(String),
    
    #[error("Deserialization error: {0}")]
    Deserialization(String),
    
    #[error("Invalid TOON format: {0}")]
    InvalidFormat(String),
    
    #[error("Type error: {0}")]
    TypeError(String),
}

/// PyO3 Result type
type PyResult<T> = Result<T, PyErr>;

/// Convert a Python object to a Rust ToonValue
fn py_to_toon_value(obj: &PyAny) -> PyResult<ToonValue> {
    if obj.is_none() {
        Ok(ToonValue::Null)
    } else if let Ok(b) = obj.extract::<bool>() {
        Ok(ToonValue::Bool(b))
    } else if let Ok(i) = obj.extract::<i64>() {
        Ok(ToonValue::Number(i as f64))
    } else if let Ok(f) = obj.extract::<f64>() {
        Ok(ToonValue::Number(f))
    } else if let Ok(s) = obj.extract::<String>() {
        Ok(ToonValue::String(s))
    } else if let Ok(list) = obj.downcast::<PyList>() {
        let mut vec = Vec::with_capacity(list.len());
        for item in list.iter() {
            vec.push(py_to_toon_value(item)?);
        }
        Ok(ToonValue::Array(vec))
    } else if let Ok(dict) = obj.downcast::<PyDict>() {
        let mut map = HashMap::with_capacity(dict.len());
        for (key, value) in dict.iter() {
            let key_str = key.extract::<String>()?;
            let value_toon = py_to_toon_value(value)?;
            map.insert(key_str, value_toon);
        }
        Ok(ToonValue::Object(map))
    } else {
        Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
            "Unsupported Python type"
        ))
    }
}

/// Convert a Rust ToonValue to a Python object
fn toon_value_to_py(py: Python<'_>, value: ToonValue) -> PyResult<PyObject> {
    match value {
        ToonValue::Null => Ok(py.None().into()),
        ToonValue::Bool(b) => Ok(b.into_py(py)),
        ToonValue::Number(n) => {
            if n.fract() == 0.0 && n >= (i64::MIN as f64) && n <= (i64::MAX as f64) {
                Ok((n as i64).into_py(py))
            } else {
                Ok(n.into_py(py))
            }
        }
        ToonValue::String(s) => Ok(s.into_py(py)),
        ToonValue::Array(arr) => {
            let list = PyList::empty(py);
            for item in arr {
                list.append(toon_value_to_py(py, item)?)?;
            }
            Ok(list.into())
        }
        ToonValue::Object(map) => {
            let dict = PyDict::new(py);
            for (k, v) in map {
                dict.set_item(k, toon_value_to_py(py, v)?)?;
            }
            Ok(dict.into())
        }
    }
}

/// Encode a Python object to TOON format
#[pyfunction]
fn encode(py: Python, obj: &PyAny) -> PyResult<String> {
    let toon_value = py_to_toon_value(obj)?;
    encoder::encode(&toon_value).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(
            format!("Failed to encode: {}", e)
        )
    })
}

/// Decode a TOON string to a Python object
#[pyfunction]
fn decode(py: Python, s: &str) -> PyResult<PyObject> {
    let toon_value = decoder::decode(s).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(
            format!("Failed to decode: {}", e)
        )
    })?;
    toon_value_to_py(py, toon_value)
}

/// Python module for TOON format encoding/decoding
#[pymodule]
fn toonify_rs(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(encode, m)?)?;
    m.add_function(wrap_pyfunction!(decode, m)?)?;
    
    // Add constants
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use pyo3::types::IntoPyDict;
    
    #[test]
    fn test_py_to_toon_value() -> PyResult<()> {
        Python::with_gil(|py| {
            // Test None
            let none = py.None();
            assert_eq!(py_to_toon_value(none)?, ToonValue::Null);
            
            // Test bool
            let py_true = true.to_object(py);
            let py_false = false.to_object(py);
            assert_eq!(py_to_toon_value(py_true.as_ref(py))?, ToonValue::Bool(true));
            assert_eq!(py_to_toon_value(py_false.as_ref(py))?, ToonValue::Bool(false));
            
            // Test number
            let py_int = 42.to_object(py);
            let py_float = 3.14.to_object(py);
            assert_eq!(py_to_toon_value(py_int.as_ref(py))?, ToonValue::Number(42.0));
            assert_eq!(py_to_toon_value(py_float.as_ref(py))?, ToonValue::Number(3.14));
            
            // Test string
            let py_str = "hello".to_object(py);
            assert_eq!(
                py_to_toon_value(py_str.as_ref(py))?,
                ToonValue::String("hello".to_string())
            );
            
            // Test list
            let py_list = vec![1, 2, 3].to_object(py);
            let expected = ToonValue::Array(vec![
                ToonValue::Number(1.0),
                ToonValue::Number(2.0),
                ToonValue::Number(3.0),
            ]);
            assert_eq!(py_to_toon_value(py_list.as_ref(py))?, expected);
            
            // Test dict
            let py_dict = [("a", 1), ("b", 2)].into_py_dict(py);
            let expected = {
                let mut map = std::collections::HashMap::new();
                map.insert("a".to_string(), ToonValue::Number(1.0));
                map.insert("b".to_string(), ToonValue::Number(2.0));
                ToonValue::Object(map)
            };
            assert_eq!(py_to_toon_value(py_dict.into())?, expected);
            
            Ok(())
        })
    }
}
