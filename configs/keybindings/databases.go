// FilePath: configs/keybindings/databases.go

// This file contains keybinding configurations for the databases panel.

package keybindings

import (
	"github.com/spf13/viper"
)

// DatabasesKeybindings contains key bindings for the databases panel
type DatabasesKeybindings struct {
	SelectDatabase string
	NavigateUp     string
	NavigateDown   string
}

// SetDatabasesDefaults sets default values for databases panel key bindings
func SetDatabasesDefaults(v *viper.Viper) {
	v.SetDefault("keybindings.databases.selectDatabase", "s")
	v.SetDefault("keybindings.databases.navigateUp", "k")
	v.SetDefault("keybindings.databases.navigateDown", "j")
}

// LoadDatabasesKeybindings loads databases panel keybinding settings from viper
func LoadDatabasesKeybindings(v *viper.Viper) DatabasesKeybindings {
	return DatabasesKeybindings{
		SelectDatabase: v.GetString("keybindings.databases.selectDatabase"),
		NavigateUp:     v.GetString("keybindings.databases.navigateUp"),
		NavigateDown:   v.GetString("keybindings.databases.navigateDown"),
	}
}
