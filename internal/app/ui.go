// FilePath: internal/app/ui.go

// ui.go handles the construction and management of the user interface components
// for the LazyTables application. It creates, arranges, and manages all the visual
// elements and their layout.

package app

import (
	"github.com/rivo/tview"
	"github.com/yuyudhan/LazyTables/internal/ui/views"
	"github.com/yuyudhan/LazyTables/pkg/logger"
)

// initUI initializes all UI components and constructs the application layout
func (app *App) initUI() {
	logger.Info("Initializing UI components")

	// Create status bar at the bottom of the screen
	app.statusBar = views.NewStatusBar(app.config.UI)

	// Create notification manager for transient messages
	app.notifications = views.NewNotificationManager(app.config.UI.NotificationDuration)

	// Create left sidebar components
	logger.Debug("Creating sidebar components")
	app.connectionsBox = views.NewConnectionsBox(app.databaseManager, app.notifications)
	app.databasesBox = views.NewDatabasesBox(app.databaseManager, app.notifications)
	app.tablesBox = views.NewTablesBox(app.databaseManager, app.notifications)

	// Create main area components
	logger.Debug("Creating main area components")
	app.queryBox = views.NewQueryBox(app.databaseManager, app.notifications, app.config.UI.EnableSyntaxHighlight)
	app.outputBox = views.NewOutputBox(app.notifications)

	// Arrange the sidebar components vertically
	logger.Debug("Constructing sidebar layout")
	app.leftSidebar = tview.NewFlex().
		SetDirection(tview.FlexRow).
		AddItem(app.connectionsBox, 0, 1, false).
		AddItem(app.databasesBox, 0, 1, false).
		AddItem(app.tablesBox, 0, 1, false)

	// Arrange the main area components vertically
	logger.Debug("Constructing main area layout")
	app.mainArea = tview.NewFlex().
		SetDirection(tview.FlexRow).
		AddItem(app.queryBox, 0, 1, false).
		AddItem(app.outputBox, 0, 4, false) // 4:1 ratio (output:query)

	// Combine sidebar and main area horizontally
	logger.Debug("Constructing main application layout")
	app.layout = tview.NewFlex().
		SetDirection(tview.FlexColumn).
		AddItem(app.leftSidebar, 0, 1, false).
		AddItem(app.mainArea, 0, 4, false) // 4:1 ratio (main:sidebar)

	// Create final layout with status bar at the bottom
	root := tview.NewFlex().
		SetDirection(tview.FlexRow).
		AddItem(app.layout, 0, 1, true).
		AddItem(app.statusBar, 1, 0, false)

	// Set the root component and enable mouse support
	app.tviewApp.SetRoot(root, true).EnableMouse(true)

	// Apply initial UI state
	app.updateStatusBar()
	app.setFocus("connections")

	// Perform an initial resize to match terminal dimensions
	app.ResizeUI()

	logger.Info("UI initialization complete")
}

// ResizeUI updates the UI layout based on the current terminal size
// This is typically called when the terminal window is resized
func (app *App) ResizeUI() {
	width, height := app.tviewApp.GetScreen().Size()
	logger.Debug("Resizing UI to dimensions:", width, "x", height)

	// Calculate proportions based on configuration percentages
	sidebarWidth := width * app.config.UI.LeftSidebarWidth / 100
	mainAreaWidth := width - sidebarWidth
	queryHeight := height * app.config.UI.TopPanelHeight / 100
	outputHeight := height - queryHeight - 1 // -1 for status bar

	// Apply calculated dimensions to the layout
	app.layout.ResizeItem(app.leftSidebar, sidebarWidth, 0)
	app.layout.ResizeItem(app.mainArea, mainAreaWidth, 0)
	app.mainArea.ResizeItem(app.queryBox, queryHeight, 0)
	app.mainArea.ResizeItem(app.outputBox, outputHeight, 0)

	logger.Debug("UI resized with dimensions - Sidebar:", sidebarWidth,
		"Main:", mainAreaWidth, "Query:", queryHeight, "Output:", outputHeight)
}

// showHelpScreen displays a modal dialog with key binding information
func (app *App) showHelpScreen() {
	logger.Debug("Displaying help screen")

	helpText := "LazyTables Help\n\n" +
		"Global Keybindings:\n" +
		"c - Focus connections box\n" +
		"C - Toggle connections box visibility\n" +
		"d - Focus databases box\n" +
		"D - Toggle databases box visibility\n" +
		"t - Focus tables box\n" +
		"T - Toggle tables box visibility\n" +
		"q - Focus query box\n" +
		"Q - Toggle query box visibility\n" +
		"o - Focus output box\n" +
		"O - Toggle output box visibility\n" +
		"? - Show this help\n" +
		"Ctrl+C - Quit application"

	// Create and configure the modal dialog
	modal := tview.NewModal().
		SetText(helpText).
		AddButtons([]string{"Close"}).
		SetDoneFunc(func(buttonIndex int, buttonLabel string) {
			// Restore the original root when closed
			logger.Debug("Closing help screen")
			app.tviewApp.SetRoot(app.tviewApp.GetRoot(), true)
		})

	// Set the modal as the application root
	app.tviewApp.SetRoot(modal, true)
}
