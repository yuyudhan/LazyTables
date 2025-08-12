// FilePath: configs/app_config.go

// This file contains application-wide settings such as connection
// storage path, database timeouts, and general behavior options.

package configs

import (
	"fmt"
	"os"
	"path/filepath"

	"github.com/mitchellh/go-homedir"
	"github.com/spf13/viper"
)

// AppConfig contains general application settings
type AppConfig struct {
	ConnectionsPath   string
	QueryHistoryLimit int
	ConnectionTimeout int
	QueryTimeout      int
	LogLevel          string
	AutoSaveInterval  int
}

// setAppDefaults sets default values for application settings
func setAppDefaults(v *viper.Viper) {
	home, err := homedir.Dir()
	if err != nil {
		home = "."
	}

	connectionsPath := filepath.Join(home, ".lazytables", "connections.json")

	v.SetDefault("app.connectionsPath", connectionsPath)
	v.SetDefault("app.queryHistoryLimit", 100)
	v.SetDefault("app.connectionTimeout", 10) // seconds
	v.SetDefault("app.queryTimeout", 30)      // seconds
	v.SetDefault("app.logLevel", "info")
	v.SetDefault("app.autoSaveInterval", 60) // seconds
}

// loadAppConfig loads application settings from viper
func loadAppConfig(v *viper.Viper, config *AppConfig) error {
	config.ConnectionsPath = v.GetString("app.connectionsPath")
	config.QueryHistoryLimit = v.GetInt("app.queryHistoryLimit")
	config.ConnectionTimeout = v.GetInt("app.connectionTimeout")
	config.QueryTimeout = v.GetInt("app.queryTimeout")
	config.LogLevel = v.GetString("app.logLevel")
	config.AutoSaveInterval = v.GetInt("app.autoSaveInterval")

	// Create connections directory if it doesn't exist
	connectionsDir := filepath.Dir(config.ConnectionsPath)
	if _, err := os.Stat(connectionsDir); os.IsNotExist(err) {
		if err := os.MkdirAll(connectionsDir, 0755); err != nil {
			return fmt.Errorf("failed to create connections directory: %w", err)
		}
	}

	return nil
}
