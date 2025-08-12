// FilePath: internal/app/status.go

// status.go handles the status bar updates in the LazyTables application.
// It reflects the current state of the application including active focus,
// connection details, and database selections.

package app

import (
	"github.com/yuyudhan/LazyTables/pkg/logger"
)

// updateStatusBar updates the status bar with current application state
// including active view, connection, database and table information.
func (app *App) updateStatusBar() {
	// Default status values when nothing is selected
	activeConnection := "No connection active"
	activeDatabase := "No DB active"
	activeTable := "No table active"

	// Get active connection information if available
	if conn := app.databaseManager.GetActiveConnection(); conn != nil {
		activeConnection = conn.Name
		logger.Debug("Status bar using active connection:", conn.Name)

		// Get active database if a connection is established
		if db := app.databaseManager.GetActiveDatabase(); db != "" {
			activeDatabase = db
			logger.Debug("Status bar using active database:", db)

			// Get active table if a database is selected
			if table := app.databaseManager.GetActiveTable(); table != "" {
				activeTable = table
				logger.Debug("Status bar using active table:", table)
			}
		}
	}

	// Update the status bar with the collected information
	logger.Debug("Updating status bar:", app.activeView, activeConnection, activeDatabase, activeTable)
	app.statusBar.Update(app.activeView, activeConnection, activeDatabase, activeTable)
}
