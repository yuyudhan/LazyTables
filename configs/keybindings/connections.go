// FilePath: configs/keybindings/connections.go

// This file contains keybinding configurations for the connections panel.

package keybindings

import (
	"github.com/spf13/viper"
)

// ConnectionsKeybindings contains key bindings for the connections panel
type ConnectionsKeybindings struct {
	AddConnection    string
	EditConnection   string
	RemoveConnection string
	TestConnection   string
	Connect          string
	NavigateUp       string
	NavigateDown     string
}

// SetConnectionsDefaults sets default values for connections panel key bindings
func SetConnectionsDefaults(v *viper.Viper) {
	v.SetDefault("keybindings.connections.addConnection", "a")
	v.SetDefault("keybindings.connections.editConnection", "e")
	v.SetDefault("keybindings.connections.removeConnection", "d")
	v.SetDefault("keybindings.connections.testConnection", "t")
	v.SetDefault("keybindings.connections.connect", "enter")
	v.SetDefault("keybindings.connections.navigateUp", "k")
	v.SetDefault("keybindings.connections.navigateDown", "j")
}

// LoadConnectionsKeybindings loads connections panel keybinding settings from viper
func LoadConnectionsKeybindings(v *viper.Viper) ConnectionsKeybindings {
	return ConnectionsKeybindings{
		AddConnection:    v.GetString("keybindings.connections.addConnection"),
		EditConnection:   v.GetString("keybindings.connections.editConnection"),
		RemoveConnection: v.GetString("keybindings.connections.removeConnection"),
		TestConnection:   v.GetString("keybindings.connections.testConnection"),
		Connect:          v.GetString("keybindings.connections.connect"),
		NavigateUp:       v.GetString("keybindings.connections.navigateUp"),
		NavigateDown:     v.GetString("keybindings.connections.navigateDown"),
	}
}
