// FilePath: configs/keybindings/output.go

// This file contains keybinding configurations for the output panel.

package keybindings

import (
	"github.com/spf13/viper"
)

// OutputKeybindings contains key bindings for the output panel
type OutputKeybindings struct {
	NavigateUp    string
	NavigateDown  string
	NavigateLeft  string
	NavigateRight string
	CopyCell      string
	CopyRow       string
	CopyTable     string
	ExportResults string
}

// SetOutputDefaults sets default values for output panel key bindings
func SetOutputDefaults(v *viper.Viper) {
	v.SetDefault("keybindings.output.navigateUp", "k")
	v.SetDefault("keybindings.output.navigateDown", "j")
	v.SetDefault("keybindings.output.navigateLeft", "h")
	v.SetDefault("keybindings.output.navigateRight", "l")
	v.SetDefault("keybindings.output.copyCell", "y")
	v.SetDefault("keybindings.output.copyRow", "Y")
	v.SetDefault("keybindings.output.copyTable", "ctrl+y")
	v.SetDefault("keybindings.output.exportResults", "E")
}

// LoadOutputKeybindings loads output panel keybinding settings from viper
func LoadOutputKeybindings(v *viper.Viper) OutputKeybindings {
	return OutputKeybindings{
		NavigateUp:    v.GetString("keybindings.output.navigateUp"),
		NavigateDown:  v.GetString("keybindings.output.navigateDown"),
		NavigateLeft:  v.GetString("keybindings.output.navigateLeft"),
		NavigateRight: v.GetString("keybindings.output.navigateRight"),
		CopyCell:      v.GetString("keybindings.output.copyCell"),
		CopyRow:       v.GetString("keybindings.output.copyRow"),
		CopyTable:     v.GetString("keybindings.output.copyTable"),
		ExportResults: v.GetString("keybindings.output.exportResults"),
	}
}
