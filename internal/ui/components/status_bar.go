// FilePath: internal/ui/components/status_bar.go

package components

import (
	"fmt"
	"time"

	tea "github.com/charmbracelet/bubbletea"
	"github.com/charmbracelet/lipgloss"
)

// Status bar styling constants
const (
	statusSeparator = " │ "
)

// PanelType represents the type of panel
type PanelType int

const (
	PanelConnections PanelType = iota
	PanelDatabases
	PanelTables
	PanelQuery
	PanelOutput
)

// String returns the string representation of the panel type
func (p PanelType) String() string {
	switch p {
	case PanelConnections:
		return "Connections"
	case PanelDatabases:
		return "Databases"
	case PanelTables:
		return "Tables"
	case PanelQuery:
		return "Query"
	case PanelOutput:
		return "Output"
	default:
		return "Unknown"
	}
}

// StatusBar represents the status bar at the bottom of the screen
type StatusBar struct {
	width            int
	height           int
	focusedPanel     PanelType
	activeConnection string
	activeDatabase   string
	activeTable      string
	clock            *time.Ticker
	currentTime      time.Time
}

// NewStatusBar creates a new status bar
func NewStatusBar() *StatusBar {
	return &StatusBar{
		focusedPanel:     PanelConnections,
		activeConnection: "No connection",
		activeDatabase:   "No DB active",
		activeTable:      "No table active",
		clock:            time.NewTicker(time.Second),
		currentTime:      time.Now(),
	}
}

// Init initializes the status bar
func (s *StatusBar) Init() tea.Cmd {
	return s.tickCmd()
}

// Update handles messages and updates the status bar
func (s *StatusBar) Update(msg tea.Msg) (*StatusBar, tea.Cmd) {
	switch msg := msg.(type) {
	case tea.KeyMsg:
		// Handle key messages if needed

	case time.Time:
		// Update the time every second
		s.currentTime = msg
		return s, s.tickCmd()

	case ConnectionSelectedMsg:
		// Update active connection
		s.activeConnection = msg.Connection
		if s.activeDatabase != "" && msg.Connection == "No connection" {
			// Reset database and table if connection is lost
			s.activeDatabase = "No DB active"
			s.activeTable = "No table active"
		}

	case DatabaseSelectedMsg:
		// Update active database
		s.activeDatabase = msg.Database
		// Reset table when database changes
		s.activeTable = "No table active"

	case TableSelectedMsg:
		// Update active table
		s.activeTable = msg.Table
	}

	return s, nil
}

// View renders the status bar
func (s *StatusBar) View() string {
	// Create styles for different sections
	baseStyle := lipgloss.NewStyle().
		Height(s.height).
		PaddingLeft(1).
		PaddingRight(1)

	// Focus indicator style
	focusStyle := baseStyle.Copy().
		Background(lipgloss.Color("12")).
		Foreground(lipgloss.Color("15"))

	// Info section style
	infoStyle := baseStyle.Copy().
		Background(lipgloss.Color("8")).
		Foreground(lipgloss.Color("15"))

	// Time section style
	timeStyle := baseStyle.Copy().
		Foreground(lipgloss.Color("7"))

	// Create focus indicator
	focusIndicator := focusStyle.Render(fmt.Sprintf("Panel: %s", s.focusedPanel))

	// Create connection info
	connectionInfo := infoStyle.Render(fmt.Sprintf("Connection: %s", s.activeConnection))

	// Create database info if a connection is active
	databaseInfo := ""
	if s.activeConnection != "No connection" {
		databaseInfo = infoStyle.Render(fmt.Sprintf("DB: %s", s.activeDatabase))
	}

	// Create table info if a database is active
	tableInfo := ""
	if s.activeDatabase != "No DB active" {
		tableInfo = infoStyle.Render(fmt.Sprintf("Table: %s", s.activeTable))
	}

	// Create current date and time
	dateTimeInfo := timeStyle.Render(
		fmt.Sprintf("%s %s",
			s.currentTime.Format("2006-01-02"),
			s.currentTime.Format("15:04:05"),
		),
	)

	// Calculate available width for left and right sections
	leftInfos := []string{focusIndicator, connectionInfo}
	if databaseInfo != "" {
		leftInfos = append(leftInfos, databaseInfo)
	}
	if tableInfo != "" {
		leftInfos = append(leftInfos, tableInfo)
	}

	// Combine left section
	leftSection := lipgloss.JoinHorizontal(lipgloss.Top, leftInfos...)

	// Ensure the right section is properly aligned
	remaining := s.width - lipgloss.Width(leftSection) - lipgloss.Width(dateTimeInfo)
	spacer := ""
	if remaining > 0 {
		spacer = baseStyle.Copy().Width(remaining).Render("")
	}

	// Combine everything
	return lipgloss.JoinHorizontal(lipgloss.Top, leftSection, spacer, dateTimeInfo)
}

// SetFocusedPanel sets the panel that is currently focused
func (s *StatusBar) SetFocusedPanel(panel PanelType) {
	s.focusedPanel = panel
}

// SetSize sets the dimensions of the status bar
func (s *StatusBar) SetSize(width, height int) {
	s.width = width
	s.height = height
}

// tickCmd returns a command that ticks the clock
func (s *StatusBar) tickCmd() tea.Cmd {
	return tea.Tick(time.Second, func(t time.Time) tea.Msg {
		return t
	})
}
