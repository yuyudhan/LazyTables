// FilePath: src/state/database.rs

use crate::{
    database::{
        connection::{Connection, ConnectionStorage},
        ConnectionConfig, ConnectionStatus, DatabaseObjectList, DatabaseType, TableMetadata,
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
    /// Database objects (tables, views, etc.)
    pub database_objects: Option<DatabaseObjectList>,
    /// Available schemas in the database
    pub schemas: Vec<String>,
    /// Currently selected schema
    pub selected_schema: Option<String>,
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
            database_objects: None,
            schemas: Vec::new(),
            selected_schema: None,
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
        connection_manager: &crate::database::ConnectionManager,
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
                                    connection_manager,
                                )
                                .await
                            }
                            _ => Err(format!(
                                "Database type {} not yet supported for table viewing",
                                connection.database_type.display_name()
                            )),
                        }
                    }
                    ConnectionStatus::Connecting => {
                        Err("Connection is still in progress".to_string())
                    }
                    ConnectionStatus::Disconnected => Err("Connection is disconnected".to_string()),
                    ConnectionStatus::Failed(error) => Err(format!("Connection failed: {}", error)),
                }
            } else {
                Err("No connection selected".to_string())
            }
        } else {
            Err("Invalid tab index".to_string())
        }
    }

    /// Load PostgreSQL table data using persistent ConnectionManager
    #[allow(clippy::too_many_arguments)]
    async fn load_postgres_table_data(
        &mut self,
        connection: &ConnectionConfig,
        table_name: &str,
        limit: usize,
        offset: usize,
        table_viewer_state: &mut TableViewerState,
        tab_idx: usize,
        connection_manager: &crate::database::ConnectionManager,
    ) -> Result<(), String> {
        // Ensure we have a persistent connection in the ConnectionManager
        connection_manager
            .connect(connection)
            .await
            .map_err(|e| format!("Failed to ensure connection: {e}"))?;

        // Get table columns using persistent connection
        let columns = connection_manager
            .get_table_columns(&connection.id, table_name)
            .await
            .map_err(|e| format!("Failed to retrieve columns: {e}"))?;

        crate::log_debug!(
            "Retrieved {} columns for table {} using persistent connection",
            columns.len(),
            table_name
        );

        // Get total row count using raw query
        let count_query = format!("SELECT COUNT(*) FROM {table_name}");
        let (_, count_rows) = connection_manager
            .execute_raw_query(&connection.id, &count_query)
            .await
            .map_err(|e| format!("Failed to get row count: {e}"))?;

        let total_rows = count_rows
            .first()
            .and_then(|row| row.first())
            .and_then(|count_str| count_str.parse::<usize>().ok())
            .unwrap_or(0);

        // Get table data using persistent connection
        let rows = connection_manager
            .get_table_data(&connection.id, table_name, limit, offset)
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

            crate::log_debug!("Assigned {} ColumnInfo structs to tab", tab.columns.len());

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

        // Connection is kept alive by ConnectionManager
        Ok(())
    }

    /// Load table metadata for the details pane using persistent ConnectionManager
    pub async fn load_table_metadata(
        &mut self,
        table_name: &str,
        selected_connection: usize,
        connection_manager: &crate::database::ConnectionManager,
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
                            // Ensure we have a persistent connection
                            connection_manager
                                .connect(&connection)
                                .await
                                .map_err(|e| format!("Failed to ensure connection: {e}"))?;

                            // Get table metadata using persistent connection
                            let metadata = connection_manager
                                .get_table_metadata(&connection.id, table_name)
                                .await
                                .map_err(|e| format!("Failed to retrieve metadata: {e}"))?;

                            self.current_table_metadata = Some(metadata);
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

    /// Update a cell in the database using persistent ConnectionManager
    pub async fn update_table_cell(
        &mut self,
        update: CellUpdate,
        selected_connection: usize,
        connection_manager: &crate::database::ConnectionManager,
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
                            self.update_postgres_cell(&connection, update, connection_manager)
                                .await
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

    /// Update a cell in PostgreSQL using persistent ConnectionManager
    async fn update_postgres_cell(
        &self,
        connection: &ConnectionConfig,
        update: CellUpdate,
        connection_manager: &crate::database::ConnectionManager,
    ) -> Result<(), String> {
        // Ensure we have a persistent connection
        connection_manager
            .connect(connection)
            .await
            .map_err(|e| format!("Failed to ensure connection: {e}"))?;

        // Build UPDATE SQL
        let mut where_clauses = Vec::new();
        for (pk_col, pk_val) in &update.primary_key_values {
            where_clauses.push(format!("{pk_col} = '{pk_val}'"));
        }

        if where_clauses.is_empty() {
            return Err("Cannot update row without primary key".to_string());
        }

        let sql = format!(
            "UPDATE {} SET {} = '{}' WHERE {}",
            update.table_name,
            update.column_name,
            update.new_value.replace("'", "''"), // Escape single quotes
            where_clauses.join(" AND ")
        );

        // Execute the SQL update using persistent connection
        connection_manager
            .execute_raw_query(&connection.id, &sql)
            .await
            .map_err(|e| format!("Failed to update cell: {e}"))?;

        Ok(())
    }

    /// Delete a row from the database using persistent ConnectionManager
    pub async fn delete_table_row(
        &mut self,
        confirmation: DeleteConfirmation,
        selected_connection: usize,
        connection_manager: &crate::database::ConnectionManager,
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
                            self.delete_postgres_row(&connection, confirmation, connection_manager)
                                .await
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

    /// Delete a row in PostgreSQL using persistent ConnectionManager
    async fn delete_postgres_row(
        &self,
        connection: &ConnectionConfig,
        confirmation: DeleteConfirmation,
        connection_manager: &crate::database::ConnectionManager,
    ) -> Result<(), String> {
        // Ensure we have a persistent connection
        connection_manager
            .connect(connection)
            .await
            .map_err(|e| format!("Failed to ensure connection: {e}"))?;

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

        // Execute the delete query using persistent connection
        connection_manager
            .execute_raw_query(&connection.id, &sql)
            .await
            .map_err(|e| format!("Failed to delete row: {e}"))?;

        Ok(())
    }

    /// Execute a query and return results using persistent ConnectionManager
    pub async fn execute_query(
        &mut self,
        query: &str,
        selected_connection: usize,
        connection_manager: &crate::database::ConnectionManager,
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
                            self.execute_postgres_query(&connection, query, connection_manager)
                                .await
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

    /// Execute a PostgreSQL query using persistent ConnectionManager
    async fn execute_postgres_query(
        &self,
        connection: &ConnectionConfig,
        query: &str,
        connection_manager: &crate::database::ConnectionManager,
    ) -> Result<(Vec<String>, Vec<Vec<String>>), String> {
        // Ensure we have a persistent connection
        connection_manager
            .connect(connection)
            .await
            .map_err(|e| format!("Failed to ensure connection: {e}"))?;

        // Execute query and get results using persistent connection
        let (columns, rows) = connection_manager
            .execute_raw_query(&connection.id, query)
            .await
            .map_err(|e| format!("Query execution failed: {e}"))?;

        Ok((columns, rows))
    }

    /// Try to connect to a specific database using ConnectionManager and return database objects
    pub async fn try_connect_to_database(
        &mut self,
        connection: &ConnectionConfig,
        connection_manager: &crate::database::ConnectionManager,
    ) -> Result<DatabaseObjectList, String> {
        // Query database objects based on database type
        match connection.database_type {
            DatabaseType::PostgreSQL => {
                // Ensure we have a persistent connection in the ConnectionManager
                connection_manager
                    .connect(connection)
                    .await
                    .map_err(|e| format!("Connection failed: {e}"))?;

                // Get database objects using persistent connection
                let objects = connection_manager
                    .list_database_objects(&connection.id)
                    .await
                    .map_err(|e| format!("Failed to retrieve database objects: {e}"))?;

                self.database_objects = Some(objects.clone());

                // Update legacy tables list with qualified names for non-public schemas
                self.tables = objects
                    .tables
                    .iter()
                    .map(|t| {
                        if t.schema.as_deref() == Some("public") || t.schema.is_none() {
                            t.name.clone()
                        } else {
                            t.qualified_name()
                        }
                    })
                    .collect();

                // Also add views and materialized views to the tables list
                for view in &objects.views {
                    if view.schema.as_deref() == Some("public") || view.schema.is_none() {
                        self.tables.push(view.name.clone());
                    } else {
                        self.tables.push(view.qualified_name());
                    }
                }

                for mat_view in &objects.materialized_views {
                    if mat_view.schema.as_deref() == Some("public") || mat_view.schema.is_none() {
                        self.tables.push(mat_view.name.clone());
                    } else {
                        self.tables.push(mat_view.qualified_name());
                    }
                }

                Ok(objects)
            }
            DatabaseType::MySQL | DatabaseType::MariaDB => {
                use crate::database::mysql::MySqlConnection;
                let mut conn = MySqlConnection::new(connection.clone());
                conn.connect()
                    .await
                    .map_err(|e| format!("Connection failed: {e}"))?;

                // For now, use legacy list_tables for MySQL
                let tables = conn
                    .list_tables()
                    .await
                    .map_err(|e| format!("Failed to retrieve tables: {e}"))?;

                let _ = conn.disconnect().await;

                // Convert to DatabaseObjectList
                let mut objects = DatabaseObjectList::default();
                for table in tables {
                    objects.tables.push(crate::database::DatabaseObject {
                        name: table.clone(),
                        schema: None,
                        object_type: crate::database::DatabaseObjectType::Table,
                        row_count: None,
                        size_bytes: None,
                        comment: None,
                    });
                }
                objects.total_count = objects.tables.len();

                self.database_objects = Some(objects.clone());
                self.tables = objects.tables.iter().map(|t| t.name.clone()).collect();

                Ok(objects)
            }
            DatabaseType::SQLite => {
                use crate::database::sqlite::SqliteConnection;
                let mut conn = SqliteConnection::new(connection.clone());
                conn.connect()
                    .await
                    .map_err(|e| format!("Connection failed: {e}"))?;

                // For now, use legacy list_tables for SQLite
                let tables = conn
                    .list_tables()
                    .await
                    .map_err(|e| format!("Failed to retrieve tables: {e}"))?;

                let _ = conn.disconnect().await;

                // Convert to DatabaseObjectList
                let mut objects = DatabaseObjectList::default();
                for table in tables {
                    objects.tables.push(crate::database::DatabaseObject {
                        name: table.clone(),
                        schema: None,
                        object_type: crate::database::DatabaseObjectType::Table,
                        row_count: None,
                        size_bytes: None,
                        comment: None,
                    });
                }
                objects.total_count = objects.tables.len();

                self.database_objects = Some(objects.clone());
                self.tables = objects.tables.iter().map(|t| t.name.clone()).collect();

                Ok(objects)
            }
            _ => Err(format!(
                "Database type {} not yet supported",
                connection.database_type.display_name()
            )),
        }
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
