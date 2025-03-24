// FilePath: internal/ui/components/connections_panel.go

package components

import (
	"fmt"
	"strconv"

	"github.com/charmbracelet/bubbles/key"
	"github.com/charmbracelet/bubbles/list"
	tea "github.com/charmbracelet/bubbletea"
	"github.com/charmbracelet/lipgloss"
)

// ConnectionItem represents a database connection in the list
type ConnectionItem struct {
	ID       string
	Name     string
	Type     string // postgres, mysql, sqlite
	Host     string
	Port     int
	Username string
	// Password omitted intentionally
	Database string
}

// FilterValue implements list.Item interface
func (c ConnectionItem) FilterValue() string {
	return c.Name
}

// Title returns the connection name for the list display
func (c ConnectionItem) Title() string {
	return c.Name
}

// Description returns the connection details for the list display
func (c ConnectionItem) Description() string {
	if c.Type == "sqlite" {
		return fmt.Sprintf("SQLite: %s", c.Database)
	}
	return fmt.Sprintf("%s: %s@%s:%d", c.Type, c.Username, c.Host, c.Port)
}

// ConnectionSelectedMsg is sent when a connection is selected
type ConnectionSelectedMsg struct {
	ConnectionID string
	Connection   string
	Type         string
}

// ConnectionDeletedMsg is sent when a connection is deleted
type ConnectionDeletedMsg struct {
	ConnectionID string
}

// ConnectionAddedMsg is sent when a connection is added
type ConnectionAddedMsg struct {
	ConnectionItem
}

// ConnectionsPanelKeyMap defines the keybindings for the connections panel
type ConnectionsPanelKeyMap struct {
	Up     key.Binding
	Down   key.Binding
	Add    key.Binding
	Delete key.Binding
	Select key.Binding
}

// DefaultConnectionsPanelKeyMap returns the default keybindings
func DefaultConnectionsPanelKeyMap() ConnectionsPanelKeyMap {
	return ConnectionsPanelKeyMap{
		Up: key.NewBinding(
			key.WithKeys("k", "up"),
			key.WithHelp("k/↑", "move up"),
		),
		Down: key.NewBinding(
			key.WithKeys("j", "down"),
			key.WithHelp("j/↓", "move down"),
		),
		Add: key.NewBinding(
			key.WithKeys("a"),
			key.WithHelp("a", "add connection"),
		),
		Delete: key.NewBinding(
			key.WithKeys("d"),
			key.WithHelp("d", "delete connection"),
		),
		Select: key.NewBinding(
			key.WithKeys("s", "enter"),
			key.WithHelp("s/enter", "select connection"),
		),
	}
}

// ConnectionsPanel manages the connections panel
type ConnectionsPanel struct {
	focused      bool
	width        int
	height       int
	list         list.Model
	keyMap       ConnectionsPanelKeyMap
	connections  []ConnectionItem
	selectedConn string
	showDialog   bool
	dialog       *InputDialog
}

// NewConnectionsPanel creates a new connections panel
func NewConnectionsPanel() *ConnectionsPanel {
	// Create a new list
	l := list.New([]list.Item{}, list.NewDefaultDelegate(), 0, 0)
	l.SetShowTitle(true)
	l.Title = "Connections"
	l.SetShowStatusBar(false)
	l.SetFilteringEnabled(false)
	l.SetShowHelp(false)

	// Set custom styles
	l.Styles.Title = lipgloss.NewStyle().
		Bold(true).
		Foreground(lipgloss.Color("12")).
		Padding(0, 1)

	// Create initial connections panel
	cp := &ConnectionsPanel{
		list:        l,
		keyMap:      DefaultConnectionsPanelKeyMap(),
		connections: []ConnectionItem{},
		showDialog:  false,
	}

	// Load connections (would normally be from storage)
	cp.loadConnections()

	return cp
}

// Init initializes the connections panel
func (c *ConnectionsPanel) Init() tea.Cmd {
	return nil
}

// Update handles messages and updates the connections panel
func (c *ConnectionsPanel) Update(msg tea.Msg) (*ConnectionsPanel, tea.Cmd) {
	var cmds []tea.Cmd

	// If dialog is active, update it first
	if c.showDialog && c.dialog != nil {
		dialog, cmd := c.dialog.Update(msg)
		if cmd != nil {
			cmds = append(cmds, cmd)
		}

		// Check if dialog was updated
		if dialog != c.dialog {
			c.dialog = dialog.(*InputDialog)
		}

		// Check for dialog result
		switch msg := msg.(type) {
		case InputDialogMsg:
			if msg.ID == "add_connection" && msg.Result.Confirmed {
				// Create new connection from dialog result
				newConn := ConnectionItem{
					ID:       fmt.Sprintf("conn_%d", len(c.connections)+1),
					Name:     msg.Result.Fields["Name"],
					Type:     msg.Result.Fields["Type"],
					Host:     msg.Result.Fields["Host"],
					Username: msg.Result.Fields["Username"],
					Database: msg.Result.Fields["Database"],
				}

				// Try to parse port
				if portStr := msg.Result.Fields["Port"]; portStr != "" {
					if port, err := strconv.Atoi(portStr); err == nil {
						newConn.Port = port
					}
				}

				// Add to connections
				c.addConnection(newConn)

				// Close dialog
				c.showDialog = false
				c.dialog = nil

				// Return message
				cmds = append(cmds, func() tea.Msg {
					return ConnectionAddedMsg{newConn}
				})
			} else {
				// Dialog was cancelled or closed
				c.showDialog = false
				c.dialog = nil
			}
		}

		// If dialog is active, don't process other messages
		if c.showDialog {
			return c, tea.Batch(cmds...)
		}
	}

	// Process other messages
	switch msg := msg.(type) {
	case tea.KeyMsg:
		if !c.focused {
			break
		}

		switch {
		case key.Matches(msg, c.keyMap.Add):
			// Show add connection dialog
			c.showAddConnectionDialog()
			return c, nil

		case key.Matches(msg, c.keyMap.Delete):
			if len(c.list.Items()) > 0 && c.list.Index() >= 0 {
				// Get selected connection
				selectedItem := c.list.Items()[c.list.Index()].(ConnectionItem)

				// Show confirmation dialog
				// For simplicity, we'll just delete directly in this example
				c.deleteConnection(selectedItem.ID)

				// Send message
				cmds = append(cmds, func() tea.Msg {
					return ConnectionDeletedMsg{ConnectionID: selectedItem.ID}
				})

				// If this was the selected connection, clear it
				if c.selectedConn == selectedItem.Name {
					c.selectedConn = ""
					cmds = append(cmds, func() tea.Msg {
						return ConnectionSelectedMsg{
							Connection: "No connection",
						}
					})
				}
			}

		case key.Matches(msg, c.keyMap.Select):
			if len(c.list.Items()) > 0 && c.list.Index() >= 0 {
				// Get selected connection
				selectedItem := c.list.Items()[c.list.Index()].(ConnectionItem)
				c.selectedConn = selectedItem.Name

				// Send message
				return c, func() tea.Msg {
					return ConnectionSelectedMsg{
						ConnectionID: selectedItem.ID,
						Connection:   selectedItem.Name,
						Type:         selectedItem.Type,
					}
				}
			}
		}
	}

	// Only pass through key events to the list if focused
	if c.focused {
		var cmd tea.Cmd
		c.list, cmd = c.list.Update(msg)
		cmds = append(cmds, cmd)
	}

	return c, tea.Batch(cmds...)
}

// View renders the connections panel
func (c *ConnectionsPanel) View() string {
	// If dialog is active, render it on top
	if c.showDialog && c.dialog != nil {
		dialogView := c.dialog.View()

		// Center the dialog
		dialogWidth, _ := c.dialog.GetSize()
		xPos := (c.width - dialogWidth) / 2
		if xPos < 0 {
			xPos = 0
		}

		return lipgloss.NewStyle().
			Width(c.width).
			Height(c.height).
			Render(lipgloss.Place(c.width, c.height, lipgloss.Center, lipgloss.Center, dialogView))
	}

	// Regular view - add border to the list
	return lipgloss.NewStyle().
		Width(c.width).
		Height(c.height).
		Border(lipgloss.NormalBorder()).
		BorderForeground(c.getBorderColor()).
		Render(c.list.View())
}

// SetSize sets the panel dimensions
func (c *ConnectionsPanel) SetSize(width, height int) {
	c.width = width
	c.height = height

	// Adjust for borders
	listWidth := width - 2
	listHeight := height - 2
	if listWidth > 0 && listHeight > 0 {
		c.list.SetSize(listWidth, listHeight)
	}

	// Adjust dialog size if active
	if c.showDialog && c.dialog != nil {
		dialogWidth := width * 3 / 4
		if dialogWidth > 60 {
			dialogWidth = 60
		} else if dialogWidth < 40 {
			dialogWidth = width - 4
		}

		c.dialog.SetSize(dialogWidth, 0) // Height will be determined by content
	}
}

// SetFocused sets whether the panel is focused
func (c *ConnectionsPanel) SetFocused(focused bool) {
	c.focused = focused
}

// getBorderColor returns the border color based on focus
func (c *ConnectionsPanel) getBorderColor() lipgloss.Color {
	if c.focused {
		return lipgloss.Color("12") // Bright blue for focused
	}
	return lipgloss.Color("8") // Gray for unfocused
}

// loadConnections loads the list of connections
func (c *ConnectionsPanel) loadConnections() {
	// TODO: Load from storage
	// For now, use sample data
	c.connections = []ConnectionItem{
		{
			ID:       "conn_1",
			Name:     "Local PostgreSQL",
			Type:     "postgres",
			Host:     "localhost",
			Port:     5432,
			Username: "postgres",
			Database: "postgres",
		},
		{
			ID:       "conn_2",
			Name:     "Dev MySQL",
			Type:     "mysql",
			Host:     "localhost",
			Port:     3306,
			Username: "root",
			Database: "mysql",
		},
		{
			ID:       "conn_3",
			Name:     "App Database",
			Type:     "sqlite",
			Database: "/path/to/app.db",
		},
	}

	// Convert to list items
	items := make([]list.Item, len(c.connections))
	for i, conn := range c.connections {
		items[i] = conn
	}

	c.list.SetItems(items)
}

// addConnection adds a new connection to the list
func (c *ConnectionsPanel) addConnection(conn ConnectionItem) {
	c.connections = append(c.connections, conn)

	// Update list items
	items := make([]list.Item, len(c.connections))
	for i, conn := range c.connections {
		items[i] = conn
	}

	c.list.SetItems(items)
}

// deleteConnection removes a connection from the list
func (c *ConnectionsPanel) deleteConnection(id string) {
	// Find the connection
	for i, conn := range c.connections {
		if conn.ID == id {
			// Remove from slice
			c.connections = append(c.connections[:i], c.connections[i+1:]...)
			break
		}
	}

	// Update list items
	items := make([]list.Item, len(c.connections))
	for i, conn := range c.connections {
		items[i] = conn
	}

	c.list.SetItems(items)
}

// showAddConnectionDialog displays the dialog for adding a connection
func (c *ConnectionsPanel) showAddConnectionDialog() {
	// Create dialog fields
	fields := []DialogField{
		{Label: "Name", Placeholder: "My Connection"},
		{Label: "Type", Placeholder: "postgres, mysql, or sqlite"},
		{Label: "Host", Placeholder: "localhost"},
		{Label: "Port", Placeholder: "5432"},
		{Label: "Username", Placeholder: "postgres"},
		{Label: "Password", Placeholder: "Enter password", IsPassword: true},
		{Label: "Database", Placeholder: "postgres"},
	}

	// Create dialog
	c.dialog = NewInputDialog("add_connection", "Add Database Connection", fields, nil)

	// Set dialog size
	dialogWidth := c.width * 3 / 4
	if dialogWidth > 60 {
		dialogWidth = 60
	} else if dialogWidth < 40 {
		dialogWidth = c.width - 4
	}

	c.dialog.SetSize(dialogWidth, 0) // Height will be determined by content

	// Show dialog
	c.showDialog = true
}
