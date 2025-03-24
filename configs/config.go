// FilePath: configs/config.go

// Package configs provides configuration management for LazyTables.
// This file contains the main configuration interface and loader.
package configs

import (
	"github.com/spf13/viper"
	"github.com/yuyudhan/LazyTables/configs/keybindings"
	"github.com/yuyudhan/LazyTables/pkg/logger"
)

// Config contains all configuration settings for the application
type Config struct {
	App         AppConfig
	UI          UIConfig
	Keybindings *keybindings.Config
}

// setDefaults sets default values for all configuration sections
func setDefaults(v *viper.Viper) {
	// App defaults
	setAppDefaults(v)
	// UI defaults
	setUIDefaults(v)
	// Keybinding defaults
	keybindings.SetDefaults(v)
}

// LoadDefaultConfig creates a configuration with default values only,
// without reading from disk
func LoadDefaultConfig() (*Config, error) {
	// Initialize viper
	v := viper.New()

	// Set default values
	setDefaults(v)

	// Create and populate config struct
	config := &Config{}

	// Load app config
	if err := loadAppConfig(v, &config.App); err != nil {
		return nil, err
	}

	// Load UI config
	if err := loadUIConfig(v, &config.UI); err != nil {
		return nil, err
	}

	// Load keybindings
	keybindingsConfig, err := keybindings.Load(v)
	if err != nil {
		return nil, err
	}
	config.Keybindings = keybindingsConfig

	logger.Info("Default configuration loaded")
	return config, nil
}
