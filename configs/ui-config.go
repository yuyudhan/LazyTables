// FilePath: configs/ui_config.go

// This file contains UI-related settings including themes, colors,
// layout proportions, and visual appearance preferences.

package configs

import (
	"github.com/spf13/viper"
)

// UIConfig contains settings for the user interface
type UIConfig struct {
	Theme                 string
	LeftSidebarWidth      int // Percentage of screen width
	TopPanelHeight        int // Percentage of main area height
	NotificationDuration  int // In seconds
	StatusBarStyle        string
	EnableSyntaxHighlight bool
	ColorScheme           ColorScheme
}

// ColorScheme contains color settings for UI elements
type ColorScheme struct {
	Background     string
	Foreground     string
	Border         string
	ActiveBorder   string
	Title          string
	StatusBar      string
	ErrorMessage   string
	SuccessMessage string
	WarningMessage string
	InfoMessage    string
}

// setUIDefaults sets default values for UI settings
func setUIDefaults(v *viper.Viper) {
	v.SetDefault("ui.theme", "dark")
	v.SetDefault("ui.leftSidebarWidth", 20)
	v.SetDefault("ui.topPanelHeight", 20)
	v.SetDefault("ui.notificationDuration", 3)
	v.SetDefault("ui.statusBarStyle", "powerlevel10k")
	v.SetDefault("ui.enableSyntaxHighlight", true)

	// Default color scheme (dark)
	v.SetDefault("ui.colorScheme.background", "#1E1E2E")
	v.SetDefault("ui.colorScheme.foreground", "#CDD6F4")
	v.SetDefault("ui.colorScheme.border", "#45475A")
	v.SetDefault("ui.colorScheme.activeBorder", "#89B4FA")
	v.SetDefault("ui.colorScheme.title", "#F5E0DC")
	v.SetDefault("ui.colorScheme.statusBar", "#181825")
	v.SetDefault("ui.colorScheme.errorMessage", "#F38BA8")
	v.SetDefault("ui.colorScheme.successMessage", "#A6E3A1")
	v.SetDefault("ui.colorScheme.warningMessage", "#F9E2AF")
	v.SetDefault("ui.colorScheme.infoMessage", "#89B4FA")
}

// loadUIConfig loads UI settings from viper
func loadUIConfig(v *viper.Viper, config *UIConfig) error {
	config.Theme = v.GetString("ui.theme")
	config.LeftSidebarWidth = v.GetInt("ui.leftSidebarWidth")
	config.TopPanelHeight = v.GetInt("ui.topPanelHeight")
	config.NotificationDuration = v.GetInt("ui.notificationDuration")
	config.StatusBarStyle = v.GetString("ui.statusBarStyle")
	config.EnableSyntaxHighlight = v.GetBool("ui.enableSyntaxHighlight")

	// Load color scheme
	config.ColorScheme = ColorScheme{
		Background:     v.GetString("ui.colorScheme.background"),
		Foreground:     v.GetString("ui.colorScheme.foreground"),
		Border:         v.GetString("ui.colorScheme.border"),
		ActiveBorder:   v.GetString("ui.colorScheme.activeBorder"),
		Title:          v.GetString("ui.colorScheme.title"),
		StatusBar:      v.GetString("ui.colorScheme.statusBar"),
		ErrorMessage:   v.GetString("ui.colorScheme.errorMessage"),
		SuccessMessage: v.GetString("ui.colorScheme.successMessage"),
		WarningMessage: v.GetString("ui.colorScheme.warningMessage"),
		InfoMessage:    v.GetString("ui.colorScheme.infoMessage"),
	}

	return nil
}
