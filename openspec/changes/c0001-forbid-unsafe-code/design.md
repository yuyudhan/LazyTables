## Context

LazyTables is a Rust TUI application (~68 `.rs` files under `src/`). The codebase currently has no `unsafe` blocks, but nothing prevents one from being introduced. Adding `#![forbid(unsafe_code)]` at both the crate-root level and in each module file gives belt-and-suspenders enforcement: the crate root covers the whole crate, while per-file attributes make the constraint visible locally and survive any future crate-splitting.

## Goals / Non-Goals

**Goals:**
- Add `#![forbid(unsafe_code)]` as the first inner attribute in every `.rs` file under `src/`
- Ensure the project compiles cleanly after the change
- Make it impossible to introduce `unsafe` without an explicit compile error

**Non-Goals:**
- Auditing upstream crates for unsafe usage
- Removing any existing code (no unsafe blocks exist)
- Adding `#![deny(unsafe_code)]` (weaker — can be overridden; `forbid` cannot)

## Decisions

**Per-file vs. crate-root only**

Placing `#![forbid(unsafe_code)]` in `lib.rs`/`main.rs` alone is sufficient for correctness. Per-file placement is chosen because:
- It makes the intent explicit at each module boundary without needing to trace back to the root
- It survives refactoring (e.g., extracting a module into its own crate)
- Cost is negligible: one line per file, no runtime impact

**Placement: first line of each file**

The attribute must precede any `use`, `mod`, or `extern crate` declarations. Files that already have doc comments (`//!`) get the attribute immediately after the last doc comment line so it reads as a file-level policy alongside the module documentation.

## Risks / Trade-offs

- **Risk**: A future dependency or macro expansion emits `unsafe`. This would be a compile error — desired behavior.
- **Risk**: Dependency crate uses `unsafe` internally. No impact; `#![forbid]` is lexically scoped to this crate's own source text.
- **Trade-off**: Per-file adds ~68 lines across the codebase. Negligible noise vs. the local clarity it provides.
