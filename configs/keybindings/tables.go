// FilePath: configs/keybindings/tables.go

// This file contains keybinding configurations for the tables panel.

package keybindings

import (
	"github.com/spf13/viper"
)

// TablesKeybindings contains key bindings for the tables panel
type TablesKeybindings struct {
	SelectTable  string
	NavigateUp   string
	NavigateDown string
}

// SetTablesDefaults sets default values for tables panel key bindings
func SetTablesDefaults(v *viper.Viper) {
	v.SetDefault("keybindings.tables.selectTable", "s")
	v.SetDefault("keybindings.tables.navigateUp", "k")
	v.SetDefault("keybindings.tables.navigateDown", "j")
}

// LoadTablesKeybindings loads tables panel keybinding settings from viper
func LoadTablesKeybindings(v *viper.Viper) TablesKeybindings {
	return TablesKeybindings{
		SelectTable:  v.GetString("keybindings.tables.selectTable"),
		NavigateUp:   v.GetString("keybindings.tables.navigateUp"),
		NavigateDown: v.GetString("keybindings.tables.navigateDown"),
	}
}
