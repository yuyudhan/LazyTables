// FilePath: internal/ui/components/tables_panel.go

package components

import (
	"fmt"

	"github.com/charmbracelet/bubbles/key"
	"github.com/charmbracelet/bubbles/list"
	tea "github.com/charmbracelet/bubbletea"
	"github.com/charmbracelet/lipgloss"
)

// TableItem represents a table in the tables list
type TableItem struct {
	Name string
}

// FilterValue implements list.Item interface
func (t TableItem) FilterValue() string {
	return t.Name
}

// Title returns the table name for the list display
func (t TableItem) Title() string {
	return t.Name
}

// Description returns an empty string (not needed for simple list)
func (t TableItem) Description() string {
	return ""
}

// TableSelectedMsg is sent when a table is selected
type TableSelectedMsg struct {
	Table string
}

// TablesPanel manages the tables panel
type TablesPanel struct {
	focused     bool
	width       int
	height      int
	list        list.Model
	keyMap      TablesPanelKeyMap
	currentDB   string
	selectedIdx int
}

// TablesPanelKeyMap defines the keybindings for the tables panel
type TablesPanelKeyMap struct {
	Up     key.Binding
	Down   key.Binding
	Select key.Binding
}

// DefaultTablesPanelKeyMap returns the default keybindings
func DefaultTablesPanelKeyMap() TablesPanelKeyMap {
	return TablesPanelKeyMap{
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
			key.WithHelp("s/enter", "select table"),
		),
	}
}

// NewTablesPanel creates a new tables panel
func NewTablesPanel() *TablesPanel {
	// Create a new list
	l := list.New([]list.Item{}, list.NewDefaultDelegate(), 0, 0)
	l.SetShowTitle(true)
	l.Title = "Tables"
	l.SetShowStatusBar(false)
	l.SetFilteringEnabled(false)
	l.SetShowHelp(false)

	// Set custom styles
	l.Styles.Title = lipgloss.NewStyle().
		Bold(true).
		Foreground(lipgloss.Color("12")).
		Padding(0, 1)

	return &TablesPanel{
		list:      l,
		keyMap:    DefaultTablesPanelKeyMap(),
		currentDB: "",
	}
}

// Init initializes the tables panel
func (t *TablesPanel) Init() tea.Cmd {
	return nil
}

// Update handles messages and updates the tables panel
func (t *TablesPanel) Update(msg tea.Msg) (*TablesPanel, tea.Cmd) {
	var cmds []tea.Cmd

	switch msg := msg.(type) {
	case tea.KeyMsg:
		if !t.focused {
			break
		}

		switch {
		case key.Matches(msg, t.keyMap.Select):
			if len(t.list.Items()) > 0 && t.list.Index() >= 0 {
				selectedItem := t.list.Items()[t.list.Index()].(TableItem)
				return t, func() tea.Msg {
					return TableSelectedMsg{Table: selectedItem.Name}
				}
			}
		}

	case DatabaseSelectedMsg:
		// Update tables when database is selected
		t.currentDB = msg.Database
		cmd := t.fetchTables(msg.Database)
		cmds = append(cmds, cmd)
	}

	// Only pass through key events to the list if focused
	if t.focused {
		var cmd tea.Cmd
		t.list, cmd = t.list.Update(msg)
		cmds = append(cmds, cmd)
	}

	return t, tea.Batch(cmds...)
}

// View renders the tables panel
func (t *TablesPanel) View() string {
	if t.currentDB == "" {
		// Show message when no database is selected
		return lipgloss.NewStyle().
			Width(t.width).
			Height(t.height).
			Border(lipgloss.NormalBorder()).
			BorderForeground(t.getBorderColor()).
			Padding(1, 1).
			Render("No database selected")
	}

	// Add border to the list view
	return lipgloss.NewStyle().
		Width(t.width).
		Height(t.height).
		Border(lipgloss.NormalBorder()).
		BorderForeground(t.getBorderColor()).
		Render(t.list.View())
}

// SetSize sets the panel dimensions
func (t *TablesPanel) SetSize(width, height int) {
	t.width = width
	t.height = height

	// Adjust for borders
	listWidth := width - 2
	listHeight := height - 2
	if listWidth > 0 && listHeight > 0 {
		t.list.SetSize(listWidth, listHeight)
	}
}

// SetFocused sets whether the panel is focused
func (t *TablesPanel) SetFocused(focused bool) {
	t.focused = focused
}

// getBorderColor returns the border color based on focus
func (t *TablesPanel) getBorderColor() lipgloss.Color {
	if t.focused {
		return lipgloss.Color("12") // Bright blue for focused
	}
	return lipgloss.Color("8") // Gray for unfocused
}

// fetchTables fetches tables for the selected database
func (t *TablesPanel) fetchTables(database string) tea.Cmd {
	return func() tea.Msg {
		// TODO: Implement actual database table fetching here
		// For now, return dummy data
		tables := []list.Item{
			TableItem{Name: fmt.Sprintf("%s_users", database)},
			TableItem{Name: fmt.Sprintf("%s_products", database)},
			TableItem{Name: fmt.Sprintf("%s_orders", database)},
			TableItem{Name: fmt.Sprintf("%s_categories", database)},
		}

		return tablesLoadedMsg{tables: tables}
	}
}

// tablesLoadedMsg is sent when tables are loaded
type tablesLoadedMsg struct {
	tables []list.Item
}

// SetTables sets the tables in the panel
func (t *TablesPanel) SetTables(tables []string) {
	items := make([]list.Item, len(tables))
	for i, table := range tables {
		items[i] = TableItem{Name: table}
	}

	t.list.SetItems(items)
}
