// FilePath: internal/db/adapter.go

package db

// Adapter interface defines common operations for database adapters
type Adapter interface {
	// Connection management
	Connect(connInfo interface{}) error
	Disconnect() error

	// Database operations
	GetDatabases() ([]string, error)
	UseDatabase(database string) error
	GetCurrentDatabase() string

	// Table operations
	GetTables() ([]string, error)
	GetTableInfo(table string) ([]ColumnInfo, error)

	// Query execution
	ExecuteQuery(query string) (*QueryResult, error)
}
