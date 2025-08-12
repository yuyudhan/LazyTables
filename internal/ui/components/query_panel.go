// FilePath: internal/ui/components/query_panel.go

package components

import (
	"github.com/charmbracelet/bubbles/key"
	"github.com/charmbracelet/bubbles/textarea"
	tea "github.com/charmbracelet/bubbletea"
	"github.com/charmbracelet/lipgloss"
	"github.com/yuyudhan/LazyTables/internal/db"
)

// QueryExecutedMsg is sent when a query is executed
type QueryExecutedMsg struct {
	Query  string
	Result *db.QueryResult
}

// QueryPanelKeyMap defines the keybindings for the query panel
type QueryPanelKeyMap struct {
	Execute key.Binding
}

// DefaultQueryPanelKeyMap returns the default keybindings
func DefaultQueryPanelKeyMap() QueryPanelKeyMap {
	return QueryPanelKeyMap{
		Execute: key.NewBinding(
			key.WithKeys("ctrl+e"),
			key.WithHelp("ctrl+e", "execute query"),
		),
	}
}

// QueryPanel represents the SQL query input panel
type QueryPanel struct {
	textarea    textarea.Model
	focused     bool
	width       int
	height      int
	keyMap      QueryPanelKeyMap
	borderColor lipgloss.Color
}

// NewQueryPanel creates a new query panel
func NewQueryPanel() *QueryPanel {
	ta := textarea.New()
	ta.Placeholder = "Type SQL query here..."
	ta.ShowLineNumbers = true
	ta.SetWidth(80)
	ta.SetHeight(10)

	return &QueryPanel{
		textarea:    ta,
		keyMap:      DefaultQueryPanelKeyMap(),
		borderColor: lipgloss.Color("8"), // Default unfocused color
	}
}

// Init initializes the query panel
func (q *QueryPanel) Init() tea.Cmd {
	return textarea.Blink
}

// Update handles messages and updates the query panel
func (q *QueryPanel) Update(msg tea.Msg) (*QueryPanel, tea.Cmd) {
	var cmds []tea.Cmd

	switch msg := msg.(type) {
	case tea.KeyMsg:
		// If not focused, don't handle any keys except tab
		if !q.focused {
			break
		}

		// Handle keys
		switch {
		case key.Matches(msg, q.keyMap.Execute):
			// Execute the query
			query := q.textarea.Value()
			if query != "" {
				cmds = append(cmds, q.executeQuery(query))
			}
		}
	}

	// Update textarea
	var cmd tea.Cmd
	q.textarea, cmd = q.textarea.Update(msg)
	cmds = append(cmds, cmd)

	return q, tea.Batch(cmds...)
}

// View renders the query panel
func (q *QueryPanel) View() string {
	borderStyle := lipgloss.NewStyle().
		BorderStyle(lipgloss.NormalBorder()).
		BorderForeground(q.getBorderColor()).
		Padding(0)

	// Render the textarea with border
	return borderStyle.
		Width(q.width).
		Height(q.height).
		Render(lipgloss.NewStyle().
			Width(q.width - 2). // Adjust for border
			Height(q.height - 2).
			Render(q.textarea.View()))
}

// SetSize sets the panel dimensions
func (q *QueryPanel) SetSize(width, height int) {
	q.width = width
	q.height = height

	// Adjust textarea size to fit within the panel
	q.textarea.SetWidth(width - 4) // Account for borders and padding
	q.textarea.SetHeight(height - 4)
}

// SetFocused sets whether the panel is focused
func (q *QueryPanel) SetFocused(focused bool) {
	q.focused = focused

	// Update the textarea's focus state
	if focused {
		q.textarea.Focus()
	} else {
		q.textarea.Blur()
	}
}

// getBorderColor returns the border color based on focus
func (q *QueryPanel) getBorderColor() lipgloss.Color {
	if q.focused {
		return lipgloss.Color("12") // Bright blue for focused
	}
	return lipgloss.Color("8") // Gray for unfocused
}

// executeQuery executes the SQL query and returns the result
func (q *QueryPanel) executeQuery(query string) tea.Cmd {
	return func() tea.Msg {
		// TODO: Replace with actual query execution against the selected database
		// For now, return a mock result

		// Mock result
		result := &db.QueryResult{
			Columns: []string{"id", "name", "value"},
			Rows: [][]interface{}{
				{1, "Row 1", 100},
				{2, "Row 2", 200},
				{3, "Row 3", 300},
			},
			Message: "3 rows returned",
		}

		return QueryExecutedMsg{
			Query:  query,
			Result: result,
		}
	}
}
