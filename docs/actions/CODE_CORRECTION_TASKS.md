# LazyTables Code Correction Tasks

## Priority: CRITICAL → HIGH → MEDIUM → LOW

---

## Task 1: Extract Database State **[CRITICAL]**

**Problem:** Database logic mixed with UI in `src/app/state.rs` (1400+ lines)

**Actions:**

1. Create `src/state/database.rs` with `DatabaseState` struct
2. Move methods: `execute_query()`, `load_table_data()`, `delete_postgres_row()`
3. Update refs in `src/app/mod.rs`, `src/commands/*.rs`

---

## Task 2: Fix Connection Pool **[CRITICAL]**

**Problem:** Pool exposed as public field

**Actions:**

1. Make pool private in `src/database/postgres.rs:15`
2. Add `execute_raw()` method to Connection trait
3. Implement for all adapters

---

## Task 3: Plugin System **[HIGH]**

**Problem:** No plugin system

**Actions:**

1. Create `src/plugins/mod.rs` with Plugin trait
2. Create `src/plugins/loader.rs` for loading from `~/.lazytables/plugins`
3. Add plugin dir creation in `main.rs`

---

## Task 4: Extract Editor State **[HIGH]**

**Problem:** Query editor mixed with AppState

**Actions:**

1. Create `src/state/editor.rs` with `EditorState`
2. Move query editing methods (lines 600-700, 1400-1450)
3. Update `src/commands/query.rs`

---

## Task 5: Lazy Loading **[HIGH]**

**Problem:** Loading entire tables into memory

**Actions:**

1. Add `VirtualScroll` to `TableTab` in `table_viewer.rs`
2. Implement `load_table_page_async()` with tokio::spawn
3. Render only viewport

---

## Task 6: Key Sequences **[MEDIUM]**

**Problem:** Fragile timing for dd, gg, yy

**Actions:**

1. Create `src/input/sequence.rs` with `KeySequenceHandler`
2. Replace timing checks in `app/mod.rs:400-500`

---

## Task 7: Transactions **[MEDIUM]**

**Problem:** No transaction support

**Actions:**

1. Add transaction methods to Connection trait
2. Implement for PostgreSQL with sqlx::Transaction

---

## Task 8: Error Messages **[MEDIUM]**

**Problem:** Generic error strings

**Actions:**

1. Expand `LazyTablesError` enum with specific variants
2. Replace `format!()` errors with typed errors

---

## Task 9: Plugin Examples **[MEDIUM]**

**Actions:**

1. Create `plugins/csv_export/` example
2. Create `plugins/query_history/` example

---

## Task 10: Performance Monitoring **[LOW]**

**Actions:**

1. Create `src/metrics/mod.rs`
2. Record render/query times
3. Display in debug mode

---

## Success Criteria

- [ ] `state.rs` < 500 lines
- [ ] Plugins load from `~/.lazytables/plugins`
- [ ] Tables load incrementally (100k+ rows)
- [ ] Key sequences work reliably
- [ ] 2+ working plugins

