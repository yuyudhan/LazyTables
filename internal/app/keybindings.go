// FilePath: internal/app/keybindings.go

// keybindings.go manages all keyboard interactions for the LazyTables application.
// It sets up global key handlers and routes key events to the appropriate actions.

package app

import (
	"github.com/gdamore/tcell/v2"
	"github.com/yuyudhan/LazyTables/pkg/logger"
)

// setupGlobalKeybindings configures the global key bindings for the application
// using the keybinding configuration from the app's config.
func (app *App) setupGlobalKeybindings() {
	logger.Info("Setting up global keybindings")

	// Set the input capture function to handle all keyboard events
	app.tviewApp.SetInputCapture(func(event *tcell.EventKey) *tcell.EventKey {
		// Get keybindings configuration from app config
		kb := app.config.Keybindings.Global

		// Handle special key combinations first
		if event.Key() == tcell.KeyCtrlC {
			logger.Info("Received quit command (Ctrl+C)")
			app.Stop()
			return nil
		}

		// Only process key events for printable runes
		if event.Key() != tcell.KeyRune {
			return event
		}

		keyPressed := string(event.Rune())

		// Process keybindings based on the pressed key
		switch keyPressed {
		case kb.Quit:
			logger.Info("Received quit command")
			app.Stop()
			return nil

		case kb.Help:
			logger.Debug("Showing help screen")
			app.showHelpScreen()
			return nil

		case kb.FocusConnections:
			logger.Debug("Focusing connections box")
			app.setFocus("connections")
			return nil

		case kb.ToggleConnectionsBox:
			logger.Debug("Toggling connections box visibility")
			app.toggleViewVisibility("connections")
			return nil

		case kb.FocusDatabases:
			logger.Debug("Focusing databases box")
			app.setFocus("databases")
			return nil

		case kb.ToggleDatabasesBox:
			logger.Debug("Toggling databases box visibility")
			app.toggleViewVisibility("databases")
			return nil

		case kb.FocusTables:
			logger.Debug("Focusing tables box")
			app.setFocus("tables")
			return nil

		case kb.ToggleTablesBox:
			logger.Debug("Toggling tables box visibility")
			app.toggleViewVisibility("tables")
			return nil

		case kb.FocusQuery:
			logger.Debug("Focusing query box")
			app.setFocus("query")
			return nil

		case kb.ToggleQueryBox:
			logger.Debug("Toggling query box visibility")
			app.toggleViewVisibility("query")
			return nil

		case kb.FocusOutput:
			logger.Debug("Focusing output box")
			app.setFocus("output")
			return nil

		case kb.ToggleOutputBox:
			logger.Debug("Toggling output box visibility")
			app.toggleViewVisibility("output")
			return nil
		}

		// Pass through any unhandled keys to the focused primitive
		return event
	})

	logger.Info("Global keybindings setup complete")
}
