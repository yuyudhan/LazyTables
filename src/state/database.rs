// FilePath: src/state/database.rs

use crate::{
    database::{
        connection::ConnectionStorage, ConnectionConfig, ConnectionStatus, DatabaseType,
        TableMetadata,
    },
    ui::components::{
        table_viewer::{CellUpdate, ColumnInfo, DeleteConfirmation},
        TableViewerState,
    },
};

/// Database-specific state separated from UI concerns
#[derive(Debug, Clone)]
pub struct DatabaseState {
    /// Connections storage
    pub connections: ConnectionStorage,
    /// Tables in the currently connected database
    pub tables: Vec<String>,
    /// Error message for table loading
    pub table_load_error: Option<String>,
    /// Current table metadata (for the details pane)
    pub current_table_metadata: Option<TableMetadata>,
}

impl DatabaseState {
    /// Create a new database state
    pub fn new() -> Self {
        let connections = ConnectionStorage::load().unwrap_or_default();

        Self {
            connections,
            tables: Vec::new(),
            table_load_error: None,
            current_table_metadata: None,
        }
    }

    /// Load table data for viewer
    pub async fn load_table_data(
        &mut self,
        table_viewer_state: &mut TableViewerState,
        selected_connection: usize,
        tab_idx: usize,
    ) -> Result<(), String> {
        if let Some(tab) = table_viewer_state.tabs.get_mut(tab_idx) {
            let table_name = tab.table_name.clone();
            let page = tab.current_page;
            let limit = tab.rows_per_page;
            let offset = page * limit;

            // Get the current connection
            if let Some(connection) = self
                .connections
                .connections
                .get(selected_connection)
                .cloned()
            {
                match &connection.status {
                    ConnectionStatus::Connected => {
                        // Load table data based on database type
                        match connection.database_type {
                            DatabaseType::PostgreSQL => {
                                self.load_postgres_table_data(
                                    &connection,
                                    &table_name,
                                    limit,
                                    offset,
                                    table_viewer_state,
                                    tab_idx,
                                )
                                .await
                            }
                            _ => Err(format!(
                                "Database type {} not yet supported for table viewing",
                                connection.database_type.display_name()
                            )),
                        }
                    }
                    _ => Err("No active database connection".to_string()),
                }
            } else {
                Err("No connection selected".to_string())
            }
        } else {
            Err("Invalid tab index".to_string())
        }
    }

    /// Load PostgreSQL table data
    async fn load_postgres_table_data(
        &mut self,
        connection: &ConnectionConfig,
        table_name: &str,
        limit: usize,
        offset: usize,
        table_viewer_state: &mut TableViewerState,
        tab_idx: usize,
    ) -> Result<(), String> {
        use crate::database::postgres::PostgresConnection;
        use crate::database::Connection;

        let mut pg_connection = PostgresConnection::new(connection.clone());
        pg_connection
            .connect()
            .await
            .map_err(|e| format!("Connection failed: {e}"))?;

        // Get table columns
        let columns = pg_connection
            .get_table_columns(table_name)
            .await
            .map_err(|e| format!("Failed to retrieve columns: {e}"))?;

        // Get total row count
        let total_rows = pg_connection
            .get_table_row_count(table_name)
            .await
            .map_err(|e| format!("Failed to get row count: {e}"))?;

        // Get table data
        let rows = pg_connection
            .get_table_data(table_name, limit, offset)
            .await
            .map_err(|e| format!("Failed to retrieve data: {e}"))?;

        // Update the tab with loaded data
        if let Some(tab) = table_viewer_state.tabs.get_mut(tab_idx) {
            // Convert columns to ColumnInfo
            tab.columns = columns
                .iter()
                .map(|col| ColumnInfo {
                    name: col.name.clone(),
                    data_type: col.data_type.to_sql(),
                    is_nullable: col.is_nullable,
                    is_primary_key: col.is_primary_key,
                    max_display_width: col.name.len().max(15),
                })
                .collect();

            // Find primary key columns
            tab.primary_key_columns = columns
                .iter()
                .enumerate()
                .filter(|(_, col)| col.is_primary_key)
                .map(|(idx, _)| idx)
                .collect();

            tab.rows = rows;
            tab.total_rows = total_rows;
            tab.loading = false;
            tab.error = None;
        }

        let _ = pg_connection.disconnect().await;

        Ok(())
    }

    /// Load table metadata for the details pane
    pub async fn load_table_metadata(
        &mut self,
        table_name: &str,
        selected_connection: usize,
    ) -> Result<(), String> {
        // Get the current connection
        if let Some(connection) = self
            .connections
            .connections
            .get(selected_connection)
            .cloned()
        {
            match &connection.status {
                ConnectionStatus::Connected => {
                    // Load metadata based on database type
                    match connection.database_type {
                        DatabaseType::PostgreSQL => {
                            use crate::database::postgres::PostgresConnection;
                            use crate::database::Connection;

                            let mut pg_connection = PostgresConnection::new(connection.clone());
                            pg_connection
                                .connect()
                                .await
                                .map_err(|e| format!("Connection failed: {e}"))?;

                            // Get table metadata
                            let metadata = pg_connection
                                .get_table_metadata(table_name)
                                .await
                                .map_err(|e| format!("Failed to retrieve metadata: {e}"))?;

                            self.current_table_metadata = Some(metadata);

                            let _ = pg_connection.disconnect().await;
                            Ok(())
                        }
                        _ => Err(format!(
                            "Database type {} not yet supported for metadata",
                            connection.database_type.display_name()
                        )),
                    }
                }
                _ => Err("No active database connection".to_string()),
            }
        } else {
            Err("No connection selected".to_string())
        }
    }

    /// Update a cell in the database
    pub async fn update_table_cell(
        &mut self,
        update: CellUpdate,
        selected_connection: usize,
    ) -> Result<(), String> {
        // Get the current connection
        if let Some(connection) = self
            .connections
            .connections
            .get(selected_connection)
            .cloned()
        {
            match &connection.status {
                ConnectionStatus::Connected => {
                    // Update cell based on database type
                    match connection.database_type {
                        DatabaseType::PostgreSQL => {
                            self.update_postgres_cell(&connection, update).await
                        }
                        _ => Err(format!(
                            "Database type {} not yet supported for cell updates",
                            connection.database_type.display_name()
                        )),
                    }
                }
                _ => Err("No active database connection".to_string()),
            }
        } else {
            Err("No connection selected".to_string())
        }
    }

    /// Update a cell in PostgreSQL
    async fn update_postgres_cell(
        &self,
        connection: &ConnectionConfig,
        update: CellUpdate,
    ) -> Result<(), String> {
        use crate::database::postgres::PostgresConnection;
        use crate::database::Connection;

        let mut pg_connection = PostgresConnection::new(connection.clone());
        pg_connection
            .connect()
            .await
            .map_err(|e| format!("Connection failed: {e}"))?;

        // Build UPDATE SQL
        let mut where_clauses = Vec::new();
        for (pk_col, pk_val) in &update.primary_key_values {
            where_clauses.push(format!("{pk_col} = '{pk_val}'"));
        }

        if where_clauses.is_empty() {
            return Err("Cannot update row without primary key".to_string());
        }

        let _sql = format!(
            "UPDATE {} SET {} = '{}' WHERE {}",
            update.table_name,
            update.column_name,
            update.new_value,
            where_clauses.join(" AND ")
        );
        //
        //         pg_connection
        //             .execute_sql(&sql)
        //             .await
        //             .map_err(|e| format!("Failed to update cell: {e}"))?;

        let _ = pg_connection.disconnect().await;

        Ok(())
    }

    /// Delete a row from the database
    pub async fn delete_table_row(
        &mut self,
        confirmation: DeleteConfirmation,
        selected_connection: usize,
    ) -> Result<(), String> {
        // Get the current connection
        if let Some(connection) = self
            .connections
            .connections
            .get(selected_connection)
            .cloned()
        {
            match &connection.status {
                ConnectionStatus::Connected => {
                    // Delete row based on database type
                    match connection.database_type {
                        DatabaseType::PostgreSQL => {
                            self.delete_postgres_row(&connection, confirmation).await
                        }
                        _ => Err(format!(
                            "Database type {} not yet supported for row deletion",
                            connection.database_type.display_name()
                        )),
                    }
                }
                _ => Err("No active database connection".to_string()),
            }
        } else {
            Err("No connection selected".to_string())
        }
    }

    /// Delete a row in PostgreSQL
    async fn delete_postgres_row(
        &self,
        connection: &ConnectionConfig,
        confirmation: DeleteConfirmation,
    ) -> Result<(), String> {
        use crate::database::postgres::PostgresConnection;
        use crate::database::Connection;

        let mut pg_connection = PostgresConnection::new(connection.clone());
        pg_connection
            .connect()
            .await
            .map_err(|e| format!("Connection failed: {e}"))?;

        // Build DELETE SQL
        let mut where_clauses = Vec::new();
        for (pk_col, pk_val) in &confirmation.primary_key_values {
            where_clauses.push(format!("{pk_col} = '{pk_val}'"));
        }

        if where_clauses.is_empty() {
            return Err("Cannot delete row without primary key".to_string());
        }

        let sql = format!(
            "DELETE FROM {} WHERE {}",
            confirmation.table_name,
            where_clauses.join(" AND ")
        );

        // Execute the delete query using the pool directly
        if let Some(pool) = &pg_connection.pool {
            sqlx::query(&sql)
                .execute(pool)
                .await
                .map_err(|e| format!("Failed to delete row: {e}"))?;
        } else {
            return Err("No database connection available".to_string());
        }

        let _ = pg_connection.disconnect().await;

        Ok(())
    }

    /// Execute a query and return results
    pub async fn execute_query(
        &mut self,
        query: &str,
        selected_connection: usize,
    ) -> Result<(Vec<String>, Vec<Vec<String>>), String> {
        // Get the current connection
        if let Some(connection) = self
            .connections
            .connections
            .get(selected_connection)
            .cloned()
        {
            match &connection.status {
                ConnectionStatus::Connected => {
                    // Execute query based on database type
                    match connection.database_type {
                        DatabaseType::PostgreSQL => {
                            self.execute_postgres_query(&connection, query).await
                        }
                        _ => Err(format!(
                            "Database type {} not yet supported for queries",
                            connection.database_type.display_name()
                        )),
                    }
                }
                _ => Err("No active database connection".to_string()),
            }
        } else {
            Err("No connection selected".to_string())
        }
    }

    /// Execute a PostgreSQL query
    async fn execute_postgres_query(
        &self,
        connection: &ConnectionConfig,
        query: &str,
    ) -> Result<(Vec<String>, Vec<Vec<String>>), String> {
        use crate::database::postgres::PostgresConnection;
        use crate::database::Connection;

        let mut pg_connection = PostgresConnection::new(connection.clone());
        pg_connection
            .connect()
            .await
            .map_err(|e| format!("Connection failed: {e}"))?;

        // Execute query and get results
        let (columns, rows) = pg_connection
            .execute_raw_query(query)
            .await
            .map_err(|e| format!("Query execution failed: {e}"))?;

        let _ = pg_connection.disconnect().await;

        Ok((columns, rows))
    }

    /// Try to connect to a specific database and return tables
    pub async fn try_connect_to_database(
        &self,
        connection: &ConnectionConfig,
    ) -> Result<Vec<String>, String> {
        use crate::database::Connection;

        // Create appropriate connection based on database type
        let mut db_connection: Box<dyn Connection> = match connection.database_type {
            DatabaseType::PostgreSQL => {
                use crate::database::postgres::PostgresConnection;
                Box::new(PostgresConnection::new(connection.clone()))
            }
            DatabaseType::MySQL => {
                use crate::database::mysql::MySqlConnection;
                Box::new(MySqlConnection::new(connection.clone()))
            }
            DatabaseType::MariaDB => {
                // MariaDB uses MySQL driver
                use crate::database::mysql::MySqlConnection;
                Box::new(MySqlConnection::new(connection.clone()))
            }
            DatabaseType::SQLite => {
                use crate::database::sqlite::SqliteConnection;
                Box::new(SqliteConnection::new(connection.clone()))
            }
            _ => {
                return Err(format!(
                    "Database type {} not yet supported",
                    connection.database_type.display_name()
                ))
            }
        };

        // Try to connect
        db_connection
            .connect()
            .await
            .map_err(|e| format!("Connection failed: {e}"))?;

        // Query actual tables from the database
        let tables = db_connection
            .list_tables()
            .await
            .map_err(|e| format!("Failed to retrieve tables: {e}"))?;

        // Clean up connection
        let _ = db_connection.disconnect().await;

        Ok(tables)
    }

    /// Load table schema into table editor
    pub async fn load_table_editor_from_database(
        &mut self,
        connection: &ConnectionConfig,
        table_name: &str,
        table_editor_state: &mut crate::ui::components::TableEditorState,
    ) -> Result<(), String> {
        use crate::database::postgres::PostgresConnection;
        use crate::database::Connection;

        let mut pg_connection = PostgresConnection::new(connection.clone());
        pg_connection
            .connect()
            .await
            .map_err(|e| format!("Connection failed: {e}"))?;

        // Query table columns from information_schema
        let columns = pg_connection
            .get_table_columns(table_name)
            .await
            .map_err(|e| format!("Failed to retrieve table columns: {e}"))?;

        // Convert TableColumn to ColumnDefinition for the editor
        use crate::ui::components::table_creator::{
            ColumnDefinition as EditorColumnDef, PostgresDataType,
        };

        let editor_columns: Vec<EditorColumnDef> = columns
            .into_iter()
            .map(|col| EditorColumnDef {
                name: col.name,
                data_type: PostgresDataType::Text, // TODO: Map DataType to PostgresDataType properly
                is_nullable: col.is_nullable,
                is_primary_key: col.is_primary_key,
                is_unique: false, // Not available in TableColumn
                default_value: col.default_value,
                check_constraint: None,
                references: None,
            })
            .collect();

        // Populate the table editor state with column information
        table_editor_state.columns = editor_columns;
        table_editor_state.original_columns = table_editor_state.columns.clone();

        let _ = pg_connection.disconnect().await;

        Ok(())
    }
}

impl Default for DatabaseState {
    fn default() -> Self {
        Self::new()
    }
}
