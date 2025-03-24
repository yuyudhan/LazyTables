// FilePath: internal/db/postgres/adapter.go

package postgres

import (
	"context"
	"database/sql"
	"fmt"
	"time"

	_ "github.com/lib/pq" // PostgreSQL driver
	"github.com/yuyudhan/LazyTables/internal/db"
	"github.com/yuyudhan/LazyTables/pkg/logger"
)

// Adapter implements the db.Adapter interface for PostgreSQL databases
type Adapter struct {
	conn         *sql.DB
	currentDB    string
	queryTimeout time.Duration
}

// ConnectionInfo holds connection parameters for PostgreSQL
type ConnectionInfo struct {
	Host     string
	Port     int
	User     string
	Password string
	Database string
	SSLMode  string
}

// NewAdapter creates a new PostgreSQL adapter instance
func NewAdapter(queryTimeout int) *Adapter {
	return &Adapter{
		queryTimeout: time.Duration(queryTimeout) * time.Second,
	}
}

// Connect establishes a connection to the PostgreSQL server
func (a *Adapter) Connect(connInfo interface{}) error {
	info, ok := connInfo.(ConnectionInfo)
	if !ok {
		return fmt.Errorf("invalid connection info type for PostgreSQL")
	}

	logger.Debug("Connecting to PostgreSQL server:", info.Host, info.Port)

	// Build connection string
	connStr := fmt.Sprintf(
		"host=%s port=%d user=%s password=%s dbname=%s sslmode=%s",
		info.Host, info.Port, info.User, info.Password,
		// Use postgres as default database for initial connection
		firstNonEmpty(info.Database, "postgres"),
		firstNonEmpty(info.SSLMode, "disable"),
	)

	// Connect to database server
	db, err := sql.Open("postgres", connStr)
	if err != nil {
		logger.Error("Failed to open PostgreSQL connection:", err)
		return fmt.Errorf("failed to connect to PostgreSQL: %w", err)
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
		logger.Error("Failed to ping PostgreSQL server:", err)
		return fmt.Errorf("failed to ping PostgreSQL server: %w", err)
	}

	a.conn = db
	if info.Database != "" {
		a.currentDB = info.Database
	}

	logger.Info("Successfully connected to PostgreSQL server")
	return nil
}

// Disconnect closes the connection to the PostgreSQL server
func (a *Adapter) Disconnect() error {
	if a.conn == nil {
		return nil
	}

	logger.Debug("Disconnecting from PostgreSQL server")
	err := a.conn.Close()
	if err != nil {
		logger.Error("Error closing PostgreSQL connection:", err)
		return fmt.Errorf("error closing PostgreSQL connection: %w", err)
	}

	a.conn = nil
	a.currentDB = ""
	logger.Info("Disconnected from PostgreSQL server")
	return nil
}

// GetCurrentDatabase returns the name of the currently selected database
func (a *Adapter) GetCurrentDatabase() string {
	return a.currentDB
}

