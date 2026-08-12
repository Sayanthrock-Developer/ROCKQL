## 2024-08-07 - [Rust String Parsing Overhead]
**Learning:** In tight parsing loops in Rust (e.g., `normalize_keywords` and `remove_numeric_separators`), using `.chars().collect::<Vec<_>>()` introduces severe performance overhead due to O(N) memory allocation per operation. Similarly, repeatedly allocating new `String` instances for word extraction and `.to_ascii_lowercase()` within loops causes significant heap traffic.

**Action:** When performing string transformations or extractions in high-frequency functions, prefer slicing directly with `&str`, checking bytes directly (e.g. `value.as_bytes()`) where possible (such as when scanning for single-byte ASCII characters), and using non-allocating comparison methods like `eq_ignore_ascii_case()` instead of allocating a lowercase string. Always avoid `.chars().collect::<Vec<_>>()` when a streaming iterator (`chars()` or `as_bytes()`) suffices.
## 2024-08-07 - [Rust String Decoding Overhead]
**Learning:** Using `.char_indices()` to search for ASCII characters (like `|` or whitespace) introduces unnecessary UTF-8 decoding overhead for every character in the string, which slows down tight parsing loops.
**Action:** Use `.match_indices()` for exact char/string matches, and `.as_bytes().iter().position(...)` for searching by byte predicates (e.g. `b.is_ascii_whitespace()`), bypassing UTF-8 decoding for ASCII boundaries.
