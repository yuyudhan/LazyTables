// FilePath: internal/app/focus.go

// focus.go manages view focus and visibility in the LazyTables UI.
// It handles switching focus between different panels and toggling
// the visibility of individual UI components.

package app

import (
	"github.com/yuyudhan/LazyTables/internal/ui"
	"github.com/yuyudhan/LazyTables/pkg/logger"
)

// setFocus sets the focus to the specified view and updates the status bar
func (app *App) setFocus(view string) {
	previousView := app.activeView
	app.activeView = view

	logger.Debug("Focus changing from", previousView, "to", view)

	switch view {
	case "connections":
		app.tviewApp.SetFocus(app.connectionsBox)
	case "databases":
		app.tviewApp.SetFocus(app.databasesBox)
	case "tables":
		app.tviewApp.SetFocus(app.tablesBox)
	case "query":
		app.tviewApp.SetFocus(app.queryBox)
	case "output":
		app.tviewApp.SetFocus(app.outputBox)
	default:
		logger.Warn("Attempted to focus unknown view:", view)
		return
	}

	// Update status bar to reflect the new focus
	app.updateStatusBar()
}

// toggleViewVisibility toggles the visibility of the specified view
// and sends a notification about the change
func (app *App) toggleViewVisibility(view string) {
	var visible bool

	switch view {
	case "connections":
		visible = app.connectionsBox.IsVisible()
		app.connectionsBox.SetVisible(!visible)
	case "databases":
		visible = app.databasesBox.IsVisible()
		app.databasesBox.SetVisible(!visible)
	case "tables":
		visible = app.tablesBox.IsVisible()
		app.tablesBox.SetVisible(!visible)
	case "query":
		visible = app.queryBox.IsVisible()
		app.queryBox.SetVisible(!visible)
	case "output":
		visible = app.outputBox.IsVisible()
		app.outputBox.SetVisible(!visible)
	default:
		logger.Warn("Attempted to toggle visibility of unknown view:", view)
		return
	}

	// Notify user about visibility change
	newState := !visible
	logger.Debug("Toggled", view, "visibility to", newState)
	app.notifications.Push("info", view+" box "+ui.VisibilityText(newState))

	// Force redraw to reflect changes immediately
	app.tviewApp.Draw()
}
