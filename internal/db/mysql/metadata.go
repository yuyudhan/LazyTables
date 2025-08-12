// FilePath: internal/db/mysql/metadata.go

package mysql

import (
	"context"
	"database/sql"
	"fmt"

	"github.com/yuyudhan/LazyTables/internal/db"
	"github.com/yuyudhan/LazyTables/pkg/logger"
)

// GetDatabases returns a list of available databases
func (a *Adapter) GetDatabases() ([]string, error) {
	if a.conn == nil {
		return nil, fmt.Errorf("not connected to MySQL server")
	}

	logger.Debug("Retrieving list of MySQL databases")

	ctx, cancel := context.WithTimeout(context.Background(), a.queryTimeout)
	defer cancel()

	// In MySQL, we can use SHOW DATABASES command
	query := "SHOW DATABASES"

	rows, err := a.conn.QueryContext(ctx, query)
	if err != nil {
		logger.Error("Failed to query MySQL databases:", err)
		return nil, fmt.Errorf("failed to query databases: %w", err)
	}
	defer rows.Close()

	var databases []string
	for rows.Next() {
		var dbName string
		if err := rows.Scan(&dbName); err != nil {
			logger.Error("Error scanning database row:", err)
			return nil, fmt.Errorf("error scanning database row: %w", err)
		}

		// Skip internal MySQL databases
		if dbName != "information_schema" && dbName != "performance_schema" &&
			dbName != "mysql" && dbName != "sys" {
			databases = append(databases, dbName)
		}
	}

	if err := rows.Err(); err != nil {
		logger.Error("Error iterating database rows:", err)
		return nil, fmt.Errorf("error iterating database rows: %w", err)
	}

	logger.Debug("Retrieved", len(databases), "MySQL databases")
	return databases, nil
}

// UseDatabase switches to the specified database
func (a *Adapter) UseDatabase(database string) error {
	if a.conn == nil {
		return fmt.Errorf("not connected to MySQL server")
	}

	logger.Debug("Switching to MySQL database:", database)

	// In MySQL, we can use the USE statement to switch databases
	ctx, cancel := context.WithTimeout(context.Background(), a.queryTimeout)
	defer cancel()

	_, err := a.conn.ExecContext(ctx, fmt.Sprintf("USE %s", database))
	if err != nil {
		logger.Error("Failed to switch to database:", database, err)
		return fmt.Errorf("failed to switch to database %s: %w", database, err)
	}

	a.currentDB = database
	logger.Info("Switched to MySQL database:", database)
	return nil
}

// GetTables returns a list of tables in the current database
func (a *Adapter) GetTables() ([]string, error) {
	if a.conn == nil {
		return nil, fmt.Errorf("not connected to MySQL server")
	}

	if a.currentDB == "" {
		return nil, fmt.Errorf("no database selected")
	}

	logger.Debug("Retrieving tables from database:", a.currentDB)

	ctx, cancel := context.WithTimeout(context.Background(), a.queryTimeout)
	defer cancel()

	// In MySQL, we can use SHOW TABLES command
	query := "SHOW TABLES"

	rows, err := a.conn.QueryContext(ctx, query)
	if err != nil {
		logger.Error("Failed to query tables:", err)
		return nil, fmt.Errorf("failed to query tables: %w", err)
	}
	defer rows.Close()

	var tables []string
	for rows.Next() {
		var tableName string
		if err := rows.Scan(&tableName); err != nil {
			logger.Error("Error scanning table row:", err)
			return nil, fmt.Errorf("error scanning table row: %w", err)
		}
		tables = append(tables, tableName)
	}

	if err := rows.Err(); err != nil {
		logger.Error("Error iterating table rows:", err)
		return nil, fmt.Errorf("error iterating table rows: %w", err)
	}

	logger.Debug("Retrieved", len(tables), "tables from database:", a.currentDB)
	return tables, nil
}

// GetTableInfo returns the column information for the specified table
func (a *Adapter) GetTableInfo(table string) ([]db.ColumnInfo, error) {
	if a.conn == nil {
		return nil, fmt.Errorf("not connected to MySQL server")
	}

	if a.currentDB == "" {
		return nil, fmt.Errorf("no database selected")
	}

	logger.Debug("Retrieving column info for table:", table)

	ctx, cancel := context.WithTimeout(context.Background(), a.queryTimeout)
	defer cancel()

	// Using information_schema.columns to get column details
	query := `
		SELECT
			column_name,
			data_type,
			is_nullable,
			column_default,
			character_maximum_length,
			numeric_precision,
			numeric_scale
		FROM
			information_schema.columns
		WHERE
			table_schema = ? AND
			table_name = ?
		ORDER BY
			ordinal_position
	`

	rows, err := a.conn.QueryContext(ctx, query, a.currentDB, table)
	if err != nil {
		logger.Error("Failed to query column info:", err)
		return nil, fmt.Errorf("failed to query column info: %w", err)
	}
	defer rows.Close()

	var columns []db.ColumnInfo
	for rows.Next() {
		var col db.ColumnInfo
		var nullable, defaultVal sql.NullString
		var charMaxLen, numPrecision, numScale sql.NullInt64

		if err := rows.Scan(
			&col.Name,
			&col.Type,
			&nullable,
			&defaultVal,
			&charMaxLen,
			&numPrecision,
			&numScale,
		); err != nil {
			logger.Error("Error scanning column row:", err)
			return nil, fmt.Errorf("error scanning column row: %w", err)
		}

		// Process nullable flag
		col.Nullable = nullable.String == "YES"

		// Add default value if present
		if defaultVal.Valid {
			col.Default = defaultVal.String
		}

		// Add type-specific information
		if charMaxLen.Valid {
			col.TypeInfo = fmt.Sprintf("(%d)", charMaxLen.Int64)
		} else if numPrecision.Valid {
			if numScale.Valid && numScale.Int64 > 0 {
				col.TypeInfo = fmt.Sprintf("(%d,%d)", numPrecision.Int64, numScale.Int64)
			} else {
				col.TypeInfo = fmt.Sprintf("(%d)", numPrecision.Int64)
			}
		}

		columns = append(columns, col)
	}

	if err := rows.Err(); err != nil {
		logger.Error("Error iterating column rows:", err)
		return nil, fmt.Errorf("error iterating column rows: %w", err)
	}

	logger.Debug("Retrieved", len(columns), "columns for table:", table)
	return columns, nil
}
