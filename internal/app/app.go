// FilePath: internal/app/app.go

// app.go serves as the core entry point and orchestrator for the LazyTables application.
// It defines the main App struct, manages application lifecycle, and coordinates
// between UI components and business logic.

package app

import (
	// "github.com/gdamore/tcell/v2"
	"github.com/rivo/tview"
	"github.com/yuyudhan/LazyTables/configs"
	"github.com/yuyudhan/LazyTables/internal/database"
	"github.com/yuyudhan/LazyTables/internal/ui/views"
	"github.com/yuyudhan/LazyTables/pkg/logger"
)

// App represents the main application instance and holds references to all UI components
// and application services. It serves as the central coordination point.
type App struct {
	// Main application components
	tviewApp      *tview.Application
	layout        *tview.Flex
	statusBar     *views.StatusBar
	notifications *views.NotificationManager

	// Left sidebar components
	leftSidebar    *tview.Flex
	connectionsBox *views.ConnectionsBox
	databasesBox   *views.DatabasesBox
	tablesBox      *views.TablesBox

	// Main area components
	mainArea  *tview.Flex
	queryBox  *views.QueryBox
	outputBox *views.OutputBox

	// Application state
	config          *configs.Config
	databaseManager *database.Manager
	activeView      string
}

// New creates and initializes a new LazyTables application instance
// with all required components and default configuration.
func New() (*App, error) {
	logger.Info("Initializing LazyTables application")

	// Load application configuration
	config, err := configs.LoadDefaultConfig()
	if err != nil {
		logger.Error("Failed to load configuration:", err)
		return nil, err
	}

	// Initialize the app with default values
	app := &App{
		tviewApp:        tview.NewApplication(),
		config:          config,
		databaseManager: database.NewManager(),
		activeView:      "connections", // Default active view
	}

	// Initialize UI components
	app.initUI()

	// Setup global key bindings
	app.setupGlobalKeybindings()

	logger.Info("Application initialization complete")
	return app, nil
}

// Run starts the application main loop and blocks until the application exits
func (app *App) Run() error {
	logger.Info("Starting LazyTables application")

	// Display welcome notification
	app.notifications.Push("info", "Welcome to LazyTables!")

	// Run the application (this blocks until app.Stop is called)
	err := app.tviewApp.Run()
	if err != nil {
		logger.Error("Application terminated with error:", err)
	}

	return err
}

// Stop gracefully shuts down the application and performs any necessary cleanup
func (app *App) Stop() {
	logger.Info("Gracefully shutting down LazyTables")

	// Perform any necessary cleanup before exit
	// (e.g., close database connections, save state)
	if app.databaseManager != nil {
		app.databaseManager.CloseAllConnections()
	}

	// Stop the tview application
	app.tviewApp.Stop()
}
