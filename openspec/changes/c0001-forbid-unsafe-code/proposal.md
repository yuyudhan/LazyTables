## Why

Unsafe Rust bypasses memory safety guarantees and can introduce undefined behavior. LazyTables has no need for `unsafe` — all FFI, raw pointer arithmetic, or platform-specific hacks are either absent or handled by upstream crates. Enforcing `#![forbid(unsafe_code)]` at the file level turns an accidental `unsafe` block into a compile error, keeping the invariant machine-enforced.

## What Changes

- `#![forbid(unsafe_code)]` added as the first inner attribute in every `.rs` source file under `src/`
- Covers 68 files: crate roots (`main.rs`, `lib.rs`) and all module files
- No behavior change; build fails at compile time if any `unsafe` is introduced

## Capabilities

### New Capabilities

- `forbid-unsafe`: Compile-time enforcement that no `unsafe` code exists anywhere in the crate

### Modified Capabilities

<!-- none — no existing spec-level requirements change -->
