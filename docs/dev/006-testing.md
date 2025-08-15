# 006 - Testing

This document covers the testing strategy, setup, and best practices for LazyTables.

## Testing Philosophy

LazyTables follows a comprehensive testing approach:

- **Unit Tests**: Test individual functions and modules in isolation
- **Integration Tests**: Test component interactions and database operations
- **TUI Tests**: Test user interface components and interactions
- **Performance Tests**: Validate performance requirements
- **End-to-End Tests**: Test complete user workflows

## Test Organization

```
tests/
├── integration/           # Integration tests
│   ├── database/         # Database adapter tests
│   ├── ui/               # UI component tests
│   └── app/              # Application flow tests
├── fixtures/             # Test data and fixtures
│   ├── databases/        # Database setup files
│   ├── configs/          # Test configurations
│   └── data/             # Sample data files
├── common/               # Shared test utilities
│   ├── mod.rs           # Common test functions
│   ├── database.rs      # Database test helpers
│   └── ui.rs            # UI test helpers
└── README.md            # Testing documentation
```

## Running Tests

### Quick Test Commands

```bash
# Run all tests
make test

# Run with verbose output
cargo test -- --nocapture

# Run specific test category
cargo test --test integration
cargo test unit_tests

# Run tests for specific module
cargo test database::postgres
cargo test ui::components
```

### Continuous Testing

```bash
# Auto-run tests on file changes
cargo watch -x test

# Run tests with coverage (requires cargo-tarpaulin)
cargo tarpaulin --out Html
```

## Unit Tests

Unit tests are located alongside the source code using Rust's built-in testing framework.

### Example Unit Test Structure

```rust
// src/database/postgres.rs
#[cfg(test)]
mod tests {
    use super::*;
    use tokio_test;

    #[test]
    fn test_connection_config_validation() {
        let config = ConnectionConfig {
            host: "localhost".to_string(),
            port: 5432,
            username: "test".to_string(),
            password: "test".to_string(),
            ..Default::default()
        };
        
        assert!(config.is_valid());
    }

    #[tokio::test]
    async fn test_query_builder() {
        let builder = QueryBuilder::new();
        let query = builder
            .select("*")
            .from("users")
            .where_clause("id = $1")
            .build();
            
        assert_eq!(query, "SELECT * FROM users WHERE id = $1");
    }
}
```

### Unit Test Guidelines

- **Test one thing**: Each test should verify a single behavior
- **Use descriptive names**: Test names should explain what's being tested
- **Arrange, Act, Assert**: Structure tests with clear phases
- **Mock external dependencies**: Use mocks for database connections, file I/O
- **Test error conditions**: Include tests for failure scenarios

## Integration Tests

Integration tests verify that components work together correctly.

### Database Integration Tests

```rust
// tests/integration/database/postgres_test.rs
use lazytables::database::{DatabaseAdapter, ConnectionConfig, DatabaseType};
use lazytables::test_helpers::database::{setup_postgres, cleanup_postgres};

#[tokio::test]
async fn test_postgres_full_workflow() {
    // Setup
    let container = setup_postgres().await.expect("Failed to start Postgres");
    let config = ConnectionConfig {
        name: "test".to_string(),
        database_type: DatabaseType::PostgreSQL,
        host: "localhost".to_string(),
        port: container.port(),
        username: "test".to_string(),
        password: "test".to_string(),
        database: Some("test_db".to_string()),
        ..Default::default()
    };

    // Create adapter and connect
    let adapter = PostgresAdapter::new();
    let connection = adapter.connect(&config).await
        .expect("Failed to connect to database");

    // Test database operations
    let databases = connection.list_databases().await
        .expect("Failed to list databases");
    assert!(!databases.is_empty());

    let tables = connection.list_tables("test_db").await
        .expect("Failed to list tables");
    // Tables may be empty for new database
    
    // Test query execution
    let result = connection.execute_query("SELECT 1 as test_column").await
        .expect("Failed to execute query");
    assert_eq!(result.rows.len(), 1);
    assert_eq!(result.columns.len(), 1);
    assert_eq!(result.columns[0].name, "test_column");

    // Cleanup
    cleanup_postgres(container).await;
}
```

### UI Integration Tests

```rust
// tests/integration/ui/components_test.rs
use lazytables::ui::components::ConnectionList;
use lazytables::app::AppState;
use ratatui::backend::TestBackend;
use ratatui::Terminal;

#[test]
fn test_connection_list_rendering() {
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();
    
    let mut app_state = AppState::default();
    app_state.connections = vec![
        Connection::new("Local Postgres", DatabaseType::PostgreSQL),
        Connection::new("Remote MySQL", DatabaseType::MySQL),
    ];
    
    terminal.draw(|f| {
        let component = ConnectionList::new(&app_state.connections);
        component.render(f.size(), f);
    }).unwrap();
    
    let buffer = terminal.backend().buffer();
    // Assert that connection names are rendered
    buffer.assert_contains("Local Postgres");
    buffer.assert_contains("Remote MySQL");
}
```

## Test Database Setup

### PostgreSQL Test Container

```rust
// tests/common/database.rs
use testcontainers::{clients::Cli, Container, Docker, images::postgres::Postgres};
use std::collections::HashMap;

pub struct PostgresContainer {
    container: Container<'static, Cli, Postgres>,
    port: u16,
}

impl PostgresContainer {
    pub fn port(&self) -> u16 {
        self.port
    }
}

pub async fn setup_postgres() -> Result<PostgresContainer, Box<dyn std::error::Error>> {
    let docker = Cli::default();
    let postgres_image = Postgres::default()
        .with_db_name("test_db")
        .with_user("test")
        .with_password("test");
        
    let container = docker.run(postgres_image);
    let port = container.get_host_port(5432).unwrap();
    
    // Wait for database to be ready
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    
    Ok(PostgresContainer { container, port })
}

pub async fn cleanup_postgres(container: PostgresContainer) {
    // Container cleanup is handled automatically by testcontainers
    drop(container);
}
```

### Test Data Fixtures

```sql
-- tests/fixtures/databases/postgres_test_data.sql
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    username VARCHAR(50) UNIQUE NOT NULL,
    email VARCHAR(100) UNIQUE NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE posts (
    id SERIAL PRIMARY KEY,
    user_id INTEGER REFERENCES users(id),
    title VARCHAR(200) NOT NULL,
    content TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

INSERT INTO users (username, email) VALUES
    ('alice', 'alice@example.com'),
    ('bob', 'bob@example.com'),
    ('charlie', 'charlie@example.com');

INSERT INTO posts (user_id, title, content) VALUES
    (1, 'First Post', 'This is Alice first post'),
    (1, 'Second Post', 'Alice writes again'),
    (2, 'Bob Introduction', 'Hello, I am Bob');
```

## TUI Testing

### Testing Terminal UI Components

```rust
// tests/integration/ui/app_test.rs
use lazytables::app::App;
use lazytables::event::{AppEvent, KeyEvent};
use ratatui::backend::TestBackend;
use ratatui::Terminal;
use crossterm::event::{KeyCode, KeyModifiers};

#[tokio::test]
async fn test_app_navigation() {
    let backend = TestBackend::new(120, 40);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut app = App::new().await;
    
    // Test initial state
    assert_eq!(app.state.current_mode, Mode::Normal);
    assert_eq!(app.state.active_pane, PaneType::Connections);
    
    // Test pane switching with Ctrl+L
    let event = AppEvent::KeyPress(KeyEvent {
        code: KeyCode::Char('l'),
        modifiers: KeyModifiers::CONTROL,
    });
    
    app.handle_event(event).await.unwrap();
    assert_eq!(app.state.active_pane, PaneType::MainContent);
    
    // Test rendering
    terminal.draw(|f| {
        app.render(f);
    }).unwrap();
    
    let buffer = terminal.backend().buffer();
    // Assert UI elements are rendered correctly
    buffer.assert_contains("Connections");
    buffer.assert_contains("Tables");
}
```

### Mock Event Testing

```rust
#[tokio::test]
async fn test_keyboard_shortcuts() {
    let mut app = App::new().await;
    
    // Test ':q' for quit in command mode
    // First enter command mode
    let cmd_event = AppEvent::KeyPress(KeyEvent {
        code: KeyCode::Char(':'),
        modifiers: KeyModifiers::NONE,
    });
    app.handle_event(cmd_event).await.unwrap();
    
    // Then type 'q'
    let q_event = AppEvent::KeyPress(KeyEvent {
        code: KeyCode::Char('q'),
        modifiers: KeyModifiers::NONE,
    });
    app.handle_event(q_event).await.unwrap();
    
    // Then press Enter to execute
    let enter_event = AppEvent::KeyPress(KeyEvent {
        code: KeyCode::Enter,
        modifiers: KeyModifiers::NONE,
    });
    let result = app.handle_event(enter_event).await;
    // App should be marked to quit
    assert!(app.should_quit());
    
    // Test 'a' for add connection (in connections pane)
    app.state.active_pane = PaneType::Connections;
    let add_event = AppEvent::KeyPress(KeyEvent {
        code: KeyCode::Char('a'),
        modifiers: KeyModifiers::NONE,
    });
    
    app.handle_event(add_event).await.unwrap();
    // Assert that add connection dialog is opened
    assert!(app.state.show_add_connection_dialog);
}
```

## Performance Testing

### Startup Time Test

```rust
// tests/integration/performance/startup_test.rs
use std::time::Instant;
use lazytables::app::App;

#[tokio::test]
async fn test_startup_performance() {
    let start = Instant::now();
    
    let _app = App::new().await;
    
    let duration = start.elapsed();
    
    // Startup should be under 100ms
    assert!(duration.as_millis() < 100, 
        "Startup took {}ms, expected < 100ms", duration.as_millis());
}
```

### Query Performance Test

```rust
#[tokio::test]
async fn test_large_result_set_performance() {
    let container = setup_postgres().await.unwrap();
    // ... setup connection
    
    // Insert large dataset
    for i in 0..10000 {
        connection.execute_query(&format!(
            "INSERT INTO test_table (data) VALUES ('{}')", i
        )).await.unwrap();
    }
    
    let start = Instant::now();
    let result = connection.execute_query("SELECT * FROM test_table").await.unwrap();
    let duration = start.elapsed();
    
    assert_eq!(result.rows.len(), 10000);
    // Query should complete within reasonable time
    assert!(duration.as_millis() < 1000);
    
    cleanup_postgres(container).await;
}
```

## Test Configuration

### Cargo.toml Test Configuration

```toml
[[test]]
name = "integration"
path = "tests/integration/mod.rs"

[dev-dependencies]
tokio-test = "0.4"
testcontainers = "0.14"
ratatui = { version = "0.20", features = ["testing"] }
crossterm = "0.26"
```

### Test Environment Variables

```bash
# Set test-specific environment variables
export RUST_TEST_TIME_INTEGRATION=1
export LAZYTABLES_TEST_MODE=1
export DATABASE_TEST_TIMEOUT=30

# Run tests with environment
cargo test
```

## Continuous Integration

### GitHub Actions Test Configuration

```yaml
# .github/workflows/test.yml
name: Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    
    services:
      postgres:
        image: postgres:16
        env:
          POSTGRES_PASSWORD: test
          POSTGRES_USER: test
          POSTGRES_DB: test_db
        ports:
          - 5432:5432
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
    
    steps:
    - uses: actions/checkout@v3
    
    - name: Setup Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
        override: true
        components: rustfmt, clippy
    
    - name: Run tests
      run: |
        cargo test --all-features
    
    - name: Run clippy
      run: cargo clippy -- -D warnings
    
    - name: Check formatting
      run: cargo fmt -- --check
```

## Test Coverage

### Measuring Coverage

```bash
# Install cargo-tarpaulin
cargo install cargo-tarpaulin

# Run coverage analysis
cargo tarpaulin --out Html --output-dir coverage

# View coverage report
open coverage/tarpaulin-report.html
```

### Coverage Goals

- **Overall**: 80%+ code coverage
- **Core modules**: 90%+ coverage
- **Database adapters**: 85%+ coverage
- **UI components**: 70%+ coverage (harder to test)

## Testing Best Practices

### General Guidelines

1. **Write tests first**: Consider TDD for new features
2. **Test edge cases**: Include boundary conditions and error cases
3. **Keep tests fast**: Unit tests should run in milliseconds
4. **Make tests deterministic**: Avoid flaky tests
5. **Clean up resources**: Always clean up test databases and files

### Database Testing

1. **Use containers**: Testcontainers for consistent environments
2. **Isolate tests**: Each test gets a fresh database
3. **Test transactions**: Verify rollback behavior
4. **Mock external services**: Don't hit production databases

### UI Testing

1. **Test components**: Focus on component behavior
2. **Mock state**: Use predictable application state
3. **Verify rendering**: Check that UI elements appear correctly
4. **Test interactions**: Verify keyboard and mouse events

This comprehensive testing strategy ensures LazyTables remains reliable, performant, and maintainable as it grows.