# toonify_rs — Migration Plan (Python → Rust Core)

## 1. Overview

The goal is to replace performance-critical parts of the original [ScrapeGraphAI/toonify](https://github.com/ScrapeGraphAI/toonify) project with a **Rust-based core**, keeping full functional compatibility (`encode` / `decode` behavior identical).

TOON format (Token-Oriented Object Notation) is designed as a compact, human-readable data serialization standard. The new fork aims to achieve:

- **10–15× faster** encoding/decoding  
- **30–50% lower memory usage**  
- Full backward compatibility with Python API  
- Optional integration into real-time systems (Kafka, LLM pipelines, etc.)

---

## 2. Original Python Structure

    toon/
├── init.py
├── encoder.py
├── decoder.py
├── tokenizer.py
├── utils.py
├── schema.py
└── tests/


### Key components
| Module | Purpose |
|--------|----------|
| `encoder.py` | Serializes Python dict/list objects into TOON strings |
| `decoder.py` | Parses TOON strings back to Python objects |
| `tokenizer.py` | Lexical splitting, token handling |
| `utils.py` | String escaping, trimming, helpers |
| `schema.py` | Structure validation |

---

## 3. Bottlenecks and Issues

1. **Interpreter overhead** — recursion and loops in pure Python.  
2. **Regex-based tokenization** — slow and memory-intensive.  
3. **No streaming** — entire payloads are loaded into memory.  
4. **GC and dynamic typing** — create significant latency for large payloads.  

---

## 4. Modules to Rewrite in Rust

| Module | Rewrite? | Reason |
|---------|-----------|--------|
| `encoder.py` | ✅ | CPU-intensive; easily replaced by Rust formatter |
| `decoder.py` | ✅ | Regex-heavy; should use deterministic parser |
| `tokenizer.py` | ✅ | Ideal candidate for finite-state lexer |
| `utils.py` | ⚙️ Partial | String escape/unescape in Rust for speed |
| `schema.py` | ❌ Keep | Validation logic; not critical for performance |

---

## 5. Target Project Structure

    toonify_rs/
    ├── py_toonify/ # Python API layer
    │ ├── init.py
    │ ├── api.py # Exposes Rust encode/decode
    │ └── schema.py # Imported from original project
    └── rust_toonify/ # Rust core
    ├── Cargo.toml
    └── src/
    ├── lib.rs # PyO3 bridge
    ├── encoder.rs # Fast serializer
    ├── decoder.rs # Parser
    ├── tokenizer.rs # DFA/lexer
    ├── utils.rs # String ops
    └── types.rs # Internal data structures


Python interface remains unchanged:
```python
from toonify import encode, decode

data = {"a": 1, "b": [2, 3]}
s = encode(data)
assert decode(s) == data
6. Rust Module Details
types.rs
Defines internal representation for TOON values:

rust
Copy code
pub enum Value {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<Value>),
    Object(Vec<(String, Value)>),
}
tokenizer.rs
Lexical tokenizer — no regex, DFA-based.

rust
Copy code
#[derive(Logos, Debug, PartialEq)]
enum Token {
    #[token(":")] Colon,
    #[token(",")] Comma,
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*")] Ident,
    #[regex(r#""([^"\\]|\\.)*""#)] String,
    #[regex(r"[0-9]+(\.[0-9]+)?")] Number,
    #[error] Error,
}
encoder.rs
Recursive traversal using fmt::Write:

Supports compact and pretty modes.

Minimal heap allocation via local buffers.

decoder.rs
Parser based on nom or manual recursive descent:

Consumes TokenStream → builds Value.

Converts back to Python via Py::from.

utils.rs
Low-level string escaping and type helpers.

7. Python Integration (PyO3)
lib.rs:

rust
Copy code
use pyo3::prelude::*;
mod encoder;
mod decoder;

#[pyfunction]
fn encode(py_obj: PyObject) -> PyResult<String> {
    let v = decoder::py_to_value(py_obj)?;
    Ok(encoder::encode_value(&v))
}

#[pyfunction]
fn decode(s: &str, py: Python<'_>) -> PyResult<PyObject> {
    let v = decoder::decode_str(s)?;
    decoder::value_to_py(py, v)
}

#[pymodule]
fn toonify_rs(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(encode, m)?)?;
    m.add_function(wrap_pyfunction!(decode, m)?)?;
    Ok(())
}
Build:

bash
Copy code
maturin develop --release
8. Equivalence Tests
tests/test_equiv.py:

python
Copy code
import json
from toonify import encode as py_encode, decode as py_decode
from toonify_rs import encode as rs_encode, decode as rs_decode

with open("examples/sample.json") as f:
    data = json.load(f)

assert py_encode(data) == rs_encode(data)
assert py_decode(py_encode(data)) == rs_decode(rs_encode(data))
9. Performance Targets
Metric	Python	Rust Target	Expected Gain
Encode speed	baseline	×10	10–15× faster
Decode speed	baseline	×8	8–12× faster
Memory usage	baseline	-40%	Less GC overhead
Tokenization	regex-based	DFA/slice-based	No heap allocations

10. Optimization Options
Zero-copy parsing: work directly with &[u8].

Arena allocation: via bumpalo for temporary nodes.

SIMD acceleration: use memchr/simdutf8 for delimiter scanning.

Parallel serialization: for large arrays.

11. Development Steps
Define formal grammar (BNF/EBNF) for TOON.

Implement tokenizer.rs using logos.

Write encoder/decoder pair with full type coverage.

Integrate PyO3 bridge (encode, decode).

Add automated tests comparing outputs to Python reference.

Benchmark vs JSON, orjson, and original Toonify.

Optional: add streaming interface (encode_iter, decode_iter).

12. Roadmap (Incremental)
Phase	Deliverable	Language
Phase 1	Rust encoder.rs + PyO3 wrapper	Rust
Phase 2	Rust decoder.rs + tokenizer	Rust
Phase 3	Replace utils (escape/unescape)	Rust
Phase 4	Full parity test suite	Python
Phase 5	Benchmarks and profiling	Rust/Python
Phase 6	Optional C API for embedding	C (via cbindgen)

13. Expected Benefits
Stable binary core usable from Python, Go, or NodeJS.

High-performance TOON processing for LLM pipelines.

Maintains full backward compatibility (encode, decode).

Ready for future extensions (streaming, schema validation).

14. Build and Test Commands
bash
Copy code
# Build Rust core and Python bindings
maturin develop --release

# Run Python test suite
pytest -v

# Run Rust unit tests
cargo test --release

# Benchmark
python benchmarks/compare_encoders.py
15. Repository Naming and Structure
bash
Copy code
toonify_rs/
├── py_toonify/
│   ├── __init__.py
│   ├── api.py
│   └── schema.py
├── rust_toonify/
│   ├── Cargo.toml
│   └── src/...
├── examples/
│   └── sample.json
├── tests/
│   └── test_equiv.py
├── README.md
└── pyproject.toml
Author: Sergey Gnezdilov
Objective: Rebuild TOONIFY as a hybrid Rust-Python module with full compatibility and significant performance gain.

yaml
Copy code

---

You can copy this content directly into  
`toonify_rs/MIGRATION_PLAN.md` and commit:

```bash
git add MIGRATION_PLAN.md
git commit -m "Add Toonify_RS migration plan (Python → Rust)"
git push origin main