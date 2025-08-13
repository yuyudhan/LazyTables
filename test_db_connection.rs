// FilePath: test_db_connection.rs

use lazytables::connections::{Connection, DatabaseType};
use lazytables::database::DatabaseConnection;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("Testing database connection...");

    // Create a test connection using the info from connections.json
    let mut connection = Connection::new("Test Connection".to_string(), DatabaseType::PostgreSQL);
    connection.host = "localhost".to_string();
    connection.port = 15432;
    connection.database = "test_db".to_string();
    connection.username = "test_user".to_string();
    connection.password = Some("test_password".to_string());
    connection.ssl_mode = false;

    // Create database connection
    let mut db_conn = DatabaseConnection::new(connection);

    // Connect to database
    println!("Connecting to database...");
    match db_conn.connect().await {
        Ok(_) => println!("✓ Connected successfully!"),
        Err(e) => {
            println!("✗ Connection failed: {}", e);
            return Ok(());
        }
    }

    // Get tables
    println!("\nFetching tables...");
    match db_conn.get_tables().await {
        Ok(tables) => {
            println!("✓ Found {} tables:", tables.len());
            for table in &tables {
                println!("  - {}", table);
            }
        }
        Err(e) => println!("✗ Failed to fetch tables: {}", e),
    }

    // Test a query
    println!("\nTesting query execution...");
    match db_conn.execute_query("SELECT 1 as test").await {
        Ok((columns, data)) => {
            println!("✓ Query executed successfully!");
            println!("  Columns: {:?}", columns);
            println!("  Data: {:?}", data);
        }
        Err(e) => println!("✗ Query failed: {}", e),
    }

    // Disconnect
    db_conn.disconnect().await;
    println!("\n✓ Disconnected from database");

    Ok(())
}

