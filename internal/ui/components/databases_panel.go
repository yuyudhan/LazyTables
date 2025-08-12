// FilePath: internal/ui/components/databases_panel.go

package components

import (
	"fmt"

	"github.com/charmbracelet/bubbles/key"
	"github.com/charmbracelet/bubbles/list"
	tea "github.com/charmbracelet/bubbletea"
	"github.com/charmbracelet/lipgloss"
)

// DatabaseItem represents a database in the databases list
type DatabaseItem struct {
	Name string
}

// FilterValue implements list.Item interface
func (d DatabaseItem) FilterValue() string {
	return d.Name
}

// Title returns the database name for the list display
func (d DatabaseItem) Title() string {
	return d.Name
}

// Description returns an empty string (not needed for simple list)
func (d DatabaseItem) Description() string {
	return ""
}

// DatabaseSelectedMsg is sent when a database is selected
type DatabaseSelectedMsg struct {
	Database string
}

// DatabasesLoadedMsg is sent when databases are loaded
type DatabasesLoadedMsg struct {
	Databases []list.Item
}

// DatabasesPanelKeyMap defines the keybindings for the databases panel
type DatabasesPanelKeyMap struct {
	Up     key.Binding
	Down   key.Binding
	Select key.Binding
}

// DefaultDatabasesPanelKeyMap returns the default keybindings
func DefaultDatabasesPanelKeyMap() DatabasesPanelKeyMap {
	return DatabasesPanelKeyMap{
		Up: key.NewBinding(
			key.WithKeys("k", "up"),
			key.WithHelp("k/↑", "move up"),
		),
		Down: key.NewBinding(
			key.WithKeys("j", "down"),
			key.WithHelp("j/↓", "move down"),
		),
		Select: key.NewBinding(
			key.WithKeys("s", "enter"),
			key.WithHelp("s/enter", "select database"),
		),
	}
}

// DatabasesPanel manages the databases panel
type DatabasesPanel struct {
	focused          bool
	width            int
	height           int
	list             list.Model
	keyMap           DatabasesPanelKeyMap
	currentConnID    string
	currentConnName  string
	databases        []string
	selectedDatabase string
}

// NewDatabasesPanel creates a new databases panel
func NewDatabasesPanel() *DatabasesPanel {
	// Create a new list
	l := list.New([]list.Item{}, list.NewDefaultDelegate(), 0, 0)
	l.SetShowTitle(true)
	l.Title = "Databases"
	l.SetShowStatusBar(false)
	l.SetFilteringEnabled(false)
	l.SetShowHelp(false)

	// Set custom styles
	l.Styles.Title = lipgloss.NewStyle().
		Bold(true).
		Foreground(lipgloss.Color("12")).
		Padding(0, 1)

	return &DatabasesPanel{
		list:      l,
		keyMap:    DefaultDatabasesPanelKeyMap(),
		databases: []string{},
	}
}

// Init initializes the databases panel
func (d *DatabasesPanel) Init() tea.Cmd {
	return nil
}

// Update handles messages and updates the databases panel
func (d *DatabasesPanel) Update(msg tea.Msg) (*DatabasesPanel, tea.Cmd) {
	var cmds []tea.Cmd

	switch msg := msg.(type) {
	case tea.KeyMsg:
		if !d.focused {
			break
		}

		switch {
		case key.Matches(msg, d.keyMap.Select):
			if len(d.list.Items()) > 0 && d.list.Index() >= 0 {
				selectedItem := d.list.Items()[d.list.Index()].(DatabaseItem)
				d.selectedDatabase = selectedItem.Name
				return d, func() tea.Msg {
					return DatabaseSelectedMsg{Database: selectedItem.Name}
				}
			}
		}

	case ConnectionSelectedMsg:
		// Clear databases when connection changes
		d.currentConnID = msg.ConnectionID
		d.currentConnName = msg.Connection
		d.selectedDatabase = ""

		if msg.Connection != "No connection" {
			// Fetch databases for the selected connection
			cmd := d.fetchDatabases(msg.ConnectionID)
			cmds = append(cmds, cmd)
		} else {
			// Clear databases list
			d.list.SetItems([]list.Item{})
		}

	case DatabasesLoadedMsg:
		// Update databases list
		d.list.SetItems(msg.Databases)

		// If we have a previously selected database, try to reselect it
		if d.selectedDatabase != "" {
			for i, item := range d.list.Items() {
				if item.(DatabaseItem).Name == d.selectedDatabase {
					d.list.Select(i)
					break
				}
			}
		}
	}

	// Only pass through key events to the list if focused
	if d.focused {
		var cmd tea.Cmd
		d.list, cmd = d.list.Update(msg)
		cmds = append(cmds, cmd)
	}

	return d, tea.Batch(cmds...)
}

// View renders the databases panel
func (d *DatabasesPanel) View() string {
	if d.currentConnName == "" || d.currentConnName == "No connection" {
		// Show message when no connection is selected
		return lipgloss.NewStyle().
			Width(d.width).
			Height(d.height).
			Border(lipgloss.NormalBorder()).
			BorderForeground(d.getBorderColor()).
			Padding(1, 1).
			Render("No connection selected")
	}

	// Add border to the list view
	return lipgloss.NewStyle().
		Width(d.width).
		Height(d.height).
		Border(lipgloss.NormalBorder()).
		BorderForeground(d.getBorderColor()).
		Render(d.list.View())
}

// SetSize sets the panel dimensions
func (d *DatabasesPanel) SetSize(width, height int) {
	d.width = width
	d.height = height

	// Adjust for borders
	listWidth := width - 2
	listHeight := height - 2
	if listWidth > 0 && listHeight > 0 {
		d.list.SetSize(listWidth, listHeight)
	}
}

// SetFocused sets whether the panel is focused
func (d *DatabasesPanel) SetFocused(focused bool) {
	d.focused = focused
}

// getBorderColor returns the border color based on focus
func (d *DatabasesPanel) getBorderColor() lipgloss.Color {
	if d.focused {
		return lipgloss.Color("12") // Bright blue for focused
	}
	return lipgloss.Color("8") // Gray for unfocused
}

// fetchDatabases fetches databases for the selected connection
func (d *DatabasesPanel) fetchDatabases(connectionID string) tea.Cmd {
	return func() tea.Msg {
		// TODO: Implement actual database fetching from the connection
		// For now, return dummy data
		databases := []list.Item{
			DatabaseItem{Name: fmt.Sprintf("db_%s_1", connectionID)},
			DatabaseItem{Name: fmt.Sprintf("db_%s_2", connectionID)},
			DatabaseItem{Name: fmt.Sprintf("db_%s_3", connectionID)},
			DatabaseItem{Name: fmt.Sprintf("db_%s_4", connectionID)},
		}

		return DatabasesLoadedMsg{Databases: databases}
	}
}
