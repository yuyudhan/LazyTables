// FilePath: configs/keybindings/global.go

// This file contains global keybinding configurations.

package keybindings

import (
	"github.com/spf13/viper"
)

// GlobalKeybindings contains global application key bindings
type GlobalKeybindings struct {
	Quit                 string
	Help                 string
	FocusConnections     string
	FocusDatabases       string
	FocusTables          string
	FocusQuery           string
	FocusOutput          string
	ToggleConnectionsBox string
	ToggleDatabasesBox   string
	ToggleTablesBox      string
	ToggleQueryBox       string
	ToggleOutputBox      string
}

// SetGlobalDefaults sets default values for global key bindings
func SetGlobalDefaults(v *viper.Viper) {
	v.SetDefault("keybindings.global.quit", "ctrl+c")
	v.SetDefault("keybindings.global.help", "?")
	v.SetDefault("keybindings.global.focusConnections", "c")
	v.SetDefault("keybindings.global.focusDatabases", "d")
	v.SetDefault("keybindings.global.focusTables", "t")
	v.SetDefault("keybindings.global.focusQuery", "q")
	v.SetDefault("keybindings.global.focusOutput", "o")
	v.SetDefault("keybindings.global.toggleConnectionsBox", "C")
	v.SetDefault("keybindings.global.toggleDatabasesBox", "D")
	v.SetDefault("keybindings.global.toggleTablesBox", "T")
	v.SetDefault("keybindings.global.toggleQueryBox", "Q")
	v.SetDefault("keybindings.global.toggleOutputBox", "O")
}

// LoadGlobalKeybindings loads global keybinding settings from viper
func LoadGlobalKeybindings(v *viper.Viper) GlobalKeybindings {
	return GlobalKeybindings{
		Quit:                 v.GetString("keybindings.global.quit"),
		Help:                 v.GetString("keybindings.global.help"),
		FocusConnections:     v.GetString("keybindings.global.focusConnections"),
		FocusDatabases:       v.GetString("keybindings.global.focusDatabases"),
		FocusTables:          v.GetString("keybindings.global.focusTables"),
		FocusQuery:           v.GetString("keybindings.global.focusQuery"),
		FocusOutput:          v.GetString("keybindings.global.focusOutput"),
		ToggleConnectionsBox: v.GetString("keybindings.global.toggleConnectionsBox"),
		ToggleDatabasesBox:   v.GetString("keybindings.global.toggleDatabasesBox"),
		ToggleTablesBox:      v.GetString("keybindings.global.toggleTablesBox"),
		ToggleQueryBox:       v.GetString("keybindings.global.toggleQueryBox"),
		ToggleOutputBox:      v.GetString("keybindings.global.toggleOutputBox"),
	}
}
