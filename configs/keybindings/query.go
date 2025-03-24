// FilePath: configs/keybindings/query.go

// This file contains keybinding configurations for the query panel.

package keybindings

import (
	"github.com/spf13/viper"
)

// QueryKeybindings contains key bindings for the query panel
type QueryKeybindings struct {
	ExecuteQuery string
	ClearQuery   string
	SaveQuery    string
	LoadQuery    string
}

// SetQueryDefaults sets default values for query panel key bindings
func SetQueryDefaults(v *viper.Viper) {
	v.SetDefault("keybindings.query.executeQuery", "ctrl+e")
	v.SetDefault("keybindings.query.clearQuery", "ctrl+l")
	v.SetDefault("keybindings.query.saveQuery", "ctrl+s")
	v.SetDefault("keybindings.query.loadQuery", "ctrl+o")
}

// LoadQueryKeybindings loads query panel keybinding settings from viper
func LoadQueryKeybindings(v *viper.Viper) QueryKeybindings {
	return QueryKeybindings{
		ExecuteQuery: v.GetString("keybindings.query.executeQuery"),
		ClearQuery:   v.GetString("keybindings.query.clearQuery"),
		SaveQuery:    v.GetString("keybindings.query.saveQuery"),
		LoadQuery:    v.GetString("keybindings.query.loadQuery"),
	}
}
