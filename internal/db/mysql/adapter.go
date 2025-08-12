// FilePath: internal/db/mysql/adapter.go

package mysql

import (
	"context"
	"database/sql"
	"fmt"
	"time"

	_ "github.com/go-sql-driver/mysql" // MySQL driver
	"github.com/yuyudhan/LazyTables/internal/db"
	"github.com/yuyudhan/LazyTables/pkg/logger"
)

// Adapter implements the db.Adapter interface for MySQL databases
type Adapter struct {
	conn         *sql.DB
	currentDB    string
	queryTimeout time.Duration
}

// ConnectionInfo holds connection parameters for MySQL
type ConnectionInfo struct {
	Host     string
	Port     int
	User     string
	Password string
	Database string
	Params   string // Additional connection parameters
}

// NewAdapter creates a new MySQL adapter instance
func NewAdapter(queryTimeout int) *Adapter {
	return &Adapter{
		queryTimeout: time.Duration(queryTimeout) * time.Second,
	}
}

// Connect establishes a connection to the MySQL server
func (a *Adapter) Connect(connInfo interface{}) error {
	info, ok := connInfo.(ConnectionInfo)
	if !ok {
		return fmt.Errorf("invalid connection info type for MySQL")
	}

	logger.Debug("Connecting to MySQL server:", info.Host, info.Port)

	// Build connection string
	// Format: username:password@tcp(host:port)/dbname?param1=value1&param2=value2
	connStr := fmt.Sprintf(
		"%s:%s@tcp(%s:%d)/%s?%s",
		info.User, info.Password, info.Host, info.Port,
		firstNonEmpty(info.Database, ""), // Empty for no database selection
		firstNonEmpty(info.Params, "parseTime=true&timeout=30s"),
	)

	// Connect to database server
	db, err := sql.Open("mysql", connStr)
	if err != nil {
		logger.Error("Failed to open MySQL connection:", err)
		return fmt.Errorf("failed to connect to MySQL: %w", err)
	}

	// Set connection pool settings
	db.SetMaxOpenConns(5)
	db.SetMaxIdleConns(3)
	db.SetConnMaxLifetime(30 * time.Minute)

	// Verify connection with ping
	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()

	if err := db.PingContext(ctx); err != nil {
		db.Close()
		logger.Error("Failed to ping MySQL server:", err)
		return fmt.Errorf("failed to ping MySQL server: %w", err)
	}

	a.conn = db
	if info.Database != "" {
		a.currentDB = info.Database
	}

	logger.Info("Successfully connected to MySQL server")
	return nil
}

// Disconnect closes the connection to the MySQL server
func (a *Adapter) Disconnect() error {
	if a.conn == nil {
		return nil
	}

	logger.Debug("Disconnecting from MySQL server")
	err := a.conn.Close()
	if err != nil {
		logger.Error("Error closing MySQL connection:", err)
		return fmt.Errorf("error closing MySQL connection: %w", err)
	}

	a.conn = nil
	a.currentDB = ""
	logger.Info("Disconnected from MySQL server")
	return nil
}

// GetCurrentDatabase returns the name of the currently selected database
func (a *Adapter) GetCurrentDatabase() string {
	return a.currentDB
}
