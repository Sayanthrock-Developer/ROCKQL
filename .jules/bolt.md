## 2024-08-07 - [Rust String Parsing Overhead]
**Learning:** In tight parsing loops in Rust (e.g., `normalize_keywords` and `remove_numeric_separators`), using `.chars().collect::<Vec<_>>()` introduces severe performance overhead due to O(N) memory allocation per operation. Similarly, repeatedly allocating new `String` instances for word extraction and `.to_ascii_lowercase()` within loops causes significant heap traffic.

**Action:** When performing string transformations or extractions in high-frequency functions, prefer slicing directly with `&str`, checking bytes directly (e.g. `value.as_bytes()`) where possible (such as when scanning for single-byte ASCII characters), and using non-allocating comparison methods like `eq_ignore_ascii_case()` instead of allocating a lowercase string. Always avoid `.chars().collect::<Vec<_>>()` when a streaming iterator (`chars()` or `as_bytes()`) suffices.
## 2026-08-12 - [Avoid String allocation during initial parsing segment split]
**Learning:** In `split_segments`, constructing `Segment` previously performed `text.to_owned()` for every split segment. This caused unnecessary memory allocation, as the parsed segment string could just borrow from the original input `&str`.

**Action:** Update parsing intermediate structs (like `Segment`) to carry string slices (`&'a str`) representing chunks of the input string rather than owning `String`s when they are only used briefly to route segments to transformation parsers.
