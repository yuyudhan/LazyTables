// FilePath: configs/keybindings/keybindings.go

// This file contains the main keybindings configuration structure
// and functions to load all keybindings.
package keybindings

import (
	"github.com/spf13/viper"
)

// Config contains key bindings for various actions
type Config struct {
	Global      GlobalKeybindings
	Connections ConnectionsKeybindings
	Databases   DatabasesKeybindings
	Tables      TablesKeybindings
	Query       QueryKeybindings
	Output      OutputKeybindings
}

// SetDefaults sets all default keybinding values
func SetDefaults(v *viper.Viper) {
	// Set defaults for all keybinding categories
	SetGlobalDefaults(v)
	SetConnectionsDefaults(v)
	SetDatabasesDefaults(v)
	SetTablesDefaults(v)
	SetQueryDefaults(v)
	SetOutputDefaults(v)
}

// Load loads all keybinding settings from viper
func Load(v *viper.Viper) (*Config, error) {
	config := &Config{}

	// Load all keybinding categories
	config.Global = LoadGlobalKeybindings(v)
	config.Connections = LoadConnectionsKeybindings(v)
	config.Databases = LoadDatabasesKeybindings(v)
	config.Tables = LoadTablesKeybindings(v)
	config.Query = LoadQueryKeybindings(v)
	config.Output = LoadOutputKeybindings(v)

	return config, nil
}
