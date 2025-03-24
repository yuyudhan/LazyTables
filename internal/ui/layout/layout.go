// FilePath: internal/ui/layout/layout.go

package layout

import (
	"github.com/charmbracelet/bubbles/help"
	tea "github.com/charmbracelet/bubbletea"
	"github.com/charmbracelet/lipgloss"
	"github.com/yuyudhan/LazyTables/internal/ui/components"
)

// Layout manages the overall layout of the application
type Layout struct {
	// Dimensions
	width  int
	height int

	// Components
	connectionsPanel *components.ConnectionsPanel
	databasesPanel   *components.DatabasesPanel
	tablesPanel      *components.TablesPanel
	queryPanel       *components.QueryPanel
	outputPanel      *components.OutputPanel
	statusBar        *components.StatusBar

	// Visibility flags
	showConnections bool
	showDatabases   bool
	showTables      bool
	showQuery       bool
	showOutput      bool

	// Currently focused panel
	focusedPanel components.PanelType

	// Help model
	help help.Model
}

// NewLayout creates a new layout
func NewLayout() *Layout {
	l := &Layout{
		showConnections: true,
		showDatabases:   true,
		showTables:      true,
		showQuery:       true,
		showOutput:      true,
		focusedPanel:    components.PanelConnections,
		help:            help.New(),
	}

	// Initialize components
	l.connectionsPanel = components.NewConnectionsPanel()
	l.databasesPanel = components.NewDatabasesPanel()
	l.tablesPanel = components.NewTablesPanel()
	l.queryPanel = components.NewQueryPanel()
	l.outputPanel = components.NewOutputPanel()
	l.statusBar = components.NewStatusBar()

	return l
}

// Init initializes the layout
func (l *Layout) Init() tea.Cmd {
	cmds := []tea.Cmd{
		l.connectionsPanel.Init(),
		l.databasesPanel.Init(),
		l.tablesPanel.Init(),
		l.queryPanel.Init(),
		l.outputPanel.Init(),
		l.statusBar.Init(),
	}

	return tea.Batch(cmds...)
}

// Update handles messages and updates the layout
func (l *Layout) Update(msg tea.Msg) (tea.Model, tea.Cmd) {
	var cmds []tea.Cmd
	var cmd tea.Cmd

	switch msg := msg.(type) {
	case components.DatabaseSelectedMsg:
		// When a database is selected, update tables panel
		l.tablesPanel, cmd = l.tablesPanel.Update(msg)
		cmds = append(cmds, cmd)

		// Update status bar
		l.statusBar, cmd = l.statusBar.Update(msg)
		cmds = append(cmds, cmd)

	case components.TableSelectedMsg:
		// When a table is selected, update status bar
		l.statusBar, cmd = l.statusBar.Update(msg)
		cmds = append(cmds, cmd)

	case components.ConnectionSelectedMsg:
		// When a connection is selected, update database panel and status bar
		l.databasesPanel, cmd = l.databasesPanel.Update(msg)
		cmds = append(cmds, cmd)

		l.statusBar, cmd = l.statusBar.Update(msg)
		cmds = append(cmds, cmd)

	case components.QueryExecutedMsg:
		// When a query is executed, update output panel
		l.outputPanel, cmd = l.outputPanel.Update(msg)
		cmds = append(cmds, cmd)
	}

	// Update focused panel based on current state
	switch l.focusedPanel {
	case components.PanelConnections:
		l.connectionsPanel, cmd = l.connectionsPanel.Update(msg)
		cmds = append(cmds, cmd)

	case components.PanelDatabases:
		l.databasesPanel, cmd = l.databasesPanel.Update(msg)
		cmds = append(cmds, cmd)

	case components.PanelTables:
		l.tablesPanel, cmd = l.tablesPanel.Update(msg)
		cmds = append(cmds, cmd)

	case components.PanelQuery:
		l.queryPanel, cmd = l.queryPanel.Update(msg)
		cmds = append(cmds, cmd)

	case components.PanelOutput:
		l.outputPanel, cmd = l.outputPanel.Update(msg)
		cmds = append(cmds, cmd)
	}

	// Always update status bar
	l.statusBar, cmd = l.statusBar.Update(msg)
	cmds = append(cmds, cmd)

	return l, tea.Batch(cmds...)
}

// View renders the layout
func (l *Layout) View() string {
	// Calculate sidebar width (20% of total width)
	sidebarWidth := l.width / 5
	mainWidth := l.width - sidebarWidth

	// Calculate panel heights
	statusHeight := 1
	availHeight := l.height - statusHeight

	// Sidebar panels height allocation (30% each)
	sidebarPanelHeight := availHeight / 3

	// Main area allocation (20% query, 80% output)
	queryHeight := int(float64(availHeight) * 0.2)
	outputHeight := availHeight - queryHeight

	// Initialize empty sidebar and main area
	sidebar := ""
	mainArea := ""

	// Render sidebar panels if visible
	if l.showConnections {
		l.connectionsPanel.SetSize(sidebarWidth, sidebarPanelHeight)
		l.connectionsPanel.SetFocused(l.focusedPanel == components.PanelConnections)
		sidebar += l.connectionsPanel.View()
	}

	if l.showDatabases {
		l.databasesPanel.SetSize(sidebarWidth, sidebarPanelHeight)
		l.databasesPanel.SetFocused(l.focusedPanel == components.PanelDatabases)
		sidebar += l.databasesPanel.View()
	}

	if l.showTables {
		l.tablesPanel.SetSize(sidebarWidth, sidebarPanelHeight)
		l.tablesPanel.SetFocused(l.focusedPanel == components.PanelTables)
		sidebar += l.tablesPanel.View()
	}

	// Ensure sidebar takes exactly 1/5 of screen
	sidebar = lipgloss.NewStyle().
		Width(sidebarWidth).
		Height(availHeight).
		Render(sidebar)

	// Render main area panels if visible
	mainAreaContent := ""

	if l.showQuery {
		l.queryPanel.SetSize(mainWidth, queryHeight)
		l.queryPanel.SetFocused(l.focusedPanel == components.PanelQuery)
		mainAreaContent += l.queryPanel.View()
	}

	if l.showOutput {
		l.outputPanel.SetSize(mainWidth, outputHeight)
		l.outputPanel.SetFocused(l.focusedPanel == components.PanelOutput)
		mainAreaContent += l.outputPanel.View()
	}

	// Ensure main area takes exactly 4/5 of screen
	mainArea = lipgloss.NewStyle().
		Width(mainWidth).
		Height(availHeight).
		Render(mainAreaContent)

	// Render status bar
	l.statusBar.SetSize(l.width, statusHeight)
	l.statusBar.SetFocusedPanel(l.focusedPanel)
	statusBar := l.statusBar.View()

	// Combine everything
	content := lipgloss.JoinHorizontal(lipgloss.Top, sidebar, mainArea)
	return lipgloss.JoinVertical(lipgloss.Left, content, statusBar)
}

// UpdateSize updates the layout dimensions
func (l *Layout) UpdateSize(width, height int) tea.Cmd {
	l.width = width
	l.height = height
	return nil
}

// SetFocus sets the focused panel
func (l *Layout) SetFocus(panel components.PanelType) {
	l.focusedPanel = panel

	// Update focus state for all panels
	l.connectionsPanel.SetFocused(panel == components.PanelConnections)
	l.databasesPanel.SetFocused(panel == components.PanelDatabases)
	l.tablesPanel.SetFocused(panel == components.PanelTables)
	l.queryPanel.SetFocused(panel == components.PanelQuery)
	l.outputPanel.SetFocused(panel == components.PanelOutput)
}

// Toggle panel visibility methods

func (l *Layout) ToggleConnections() {
	l.showConnections = !l.showConnections
}

func (l *Layout) ToggleDatabases() {
	l.showDatabases = !l.showDatabases
}

func (l *Layout) ToggleTables() {
	l.showTables = !l.showTables
}

func (l *Layout) ToggleQuery() {
	l.showQuery = !l.showQuery
}

func (l *Layout) ToggleOutput() {
	l.showOutput = !l.showOutput
}
