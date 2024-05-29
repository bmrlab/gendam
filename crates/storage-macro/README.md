# How to use
```rust
use storage::*;
use storage_macro::*;

#[derive(StorageTrait)]
pub struct VideoHandler {
  // must have one
  storage: Storage,
  // or
  // library: Library
}
```
than, trait check `crates/storage/src/traits.rs`
