## ADDED Requirements

### Requirement: All source files forbid unsafe code

Every `.rs` file under `src/` MUST carry `#![forbid(unsafe_code)]` as an inner attribute at the top of the file, before any items.

#### Scenario: Crate compiles with the attribute present

- **WHEN** the developer runs `cargo build` or `cargo check`
- **THEN** the build succeeds with zero errors or warnings related to the new attribute

#### Scenario: Introducing unsafe code causes a compile error

- **WHEN** any `unsafe` block, `unsafe fn`, or `unsafe impl` is added anywhere in `src/`
- **THEN** the compiler emits a hard error (`E0453` / `unsafe_code` lint at `forbid` level) and the build fails

#### Scenario: Attribute is the first inner attribute in each file

- **WHEN** a source file has no leading doc comments
- **THEN** `#![forbid(unsafe_code)]` is the very first line
- **WHEN** a source file begins with `//!` module-level doc comments
- **THEN** `#![forbid(unsafe_code)]` appears immediately after the last `//!` line and before any `use` or `mod` declarations
