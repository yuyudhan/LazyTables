// FilePath: internal/db/mysql/query.go

package mysql

import (
	"context"
	"database/sql"
	"fmt"
	"strings"

	"github.com/yuyudhan/LazyTables/internal/db"
	"github.com/yuyudhan/LazyTables/pkg/logger"
)

// ExecuteQuery executes the given SQL query and returns the results
func (a *Adapter) ExecuteQuery(query string) (*db.QueryResult, error) {
	if a.conn == nil {
		return nil, fmt.Errorf("not connected to MySQL server")
	}

	logger.Debug("Executing query:", query)

	ctx, cancel := context.WithTimeout(context.Background(), a.queryTimeout)
	defer cancel()

	// Check if the query is a SELECT statement or similar that returns rows
	queryType := getQueryType(query)
	if queryType == "SELECT" || queryType == "SHOW" || queryType == "EXPLAIN" ||
		queryType == "DESCRIBE" || queryType == "DESC" {
		// Query that returns rows
		rows, err := a.conn.QueryContext(ctx, query)
		if err != nil {
			logger.Error("Failed to execute query:", err)
			return nil, fmt.Errorf("failed to execute query: %w", err)
		}
		defer rows.Close()

		return processQueryRows(rows)
	} else {
		// Query that doesn't return rows (UPDATE, INSERT, DELETE, etc.)
		result, err := a.conn.ExecContext(ctx, query)
		if err != nil {
			logger.Error("Failed to execute statement:", err)
			return nil, fmt.Errorf("failed to execute statement: %w", err)
		}

		affected, _ := result.RowsAffected()
		lastID, _ := result.LastInsertId()

		var message string
		if lastID > 0 {
			message = fmt.Sprintf("%d rows affected, last insert ID: %d", affected, lastID)
		} else {
			message = fmt.Sprintf("%d rows affected", affected)
		}

		queryResult := &db.QueryResult{
			Columns: []string{"Result"},
			Rows:    [][]interface{}{{message}},
			Message: message,
		}

		logger.Info("Query executed successfully,", affected, "rows affected")
		return queryResult, nil
	}
}

// processQueryRows processes SQL rows into a QueryResult
func processQueryRows(rows *sql.Rows) (*db.QueryResult, error) {
	// Get column names
	columns, err := rows.Columns()
	if err != nil {
		logger.Error("Failed to get column names:", err)
		return nil, fmt.Errorf("failed to get column names: %w", err)
	}

	// Prepare result
	result := &db.QueryResult{
		Columns: columns,
		Rows:    [][]interface{}{},
	}

	// Prepare scan targets
	scanArgs := make([]interface{}, len(columns))
	values := make([]interface{}, len(columns))
	for i := range values {
		scanArgs[i] = &values[i]
	}

	// Scan rows
	rowCount := 0
	for rows.Next() {
		err := rows.Scan(scanArgs...)
		if err != nil {
			logger.Error("Failed to scan row:", err)
			return nil, fmt.Errorf("failed to scan row: %w", err)
		}

		// Convert any nil values or binary data to appropriate representation
		row := make([]interface{}, len(columns))
		for i, v := range values {
			if v == nil {
				row[i] = "NULL"
			} else {
				switch vt := v.(type) {
				case []byte:
					// Try to convert []byte to string, but handle binary data
					if isBinary(vt) {
						row[i] = fmt.Sprintf("[BINARY DATA %d bytes]", len(vt))
					} else {
						row[i] = string(vt)
					}
				default:
					row[i] = v
				}
			}
		}

		result.Rows = append(result.Rows, row)
		rowCount++
	}

	if err := rows.Err(); err != nil {
		logger.Error("Error iterating rows:", err)
		return nil, fmt.Errorf("error iterating rows: %w", err)
	}

	result.Message = fmt.Sprintf("%d rows returned", rowCount)
	logger.Info("Query executed successfully,", rowCount, "rows returned")
	return result, nil
}
