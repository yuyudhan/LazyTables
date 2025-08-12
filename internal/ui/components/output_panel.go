// FilePath: internal/ui/components/output_panel.go

package components

import (
	"fmt"
	"strings"

	"github.com/charmbracelet/bubbles/key"
	"github.com/charmbracelet/bubbles/viewport"
	tea "github.com/charmbracelet/bubbletea"
	"github.com/charmbracelet/lipgloss"
	"github.com/yuyudhan/LazyTables/internal/db"
)

// OutputPanelKeyMap defines the keybindings for the output panel
type OutputPanelKeyMap struct {
	Up    key.Binding
	Down  key.Binding
	Left  key.Binding
	Right key.Binding
}

// DefaultOutputPanelKeyMap returns the default keybindings
func DefaultOutputPanelKeyMap() OutputPanelKeyMap {
	return OutputPanelKeyMap{
		Up: key.NewBinding(
			key.WithKeys("k", "up"),
			key.WithHelp("k/↑", "move up"),
		),
		Down: key.NewBinding(
			key.WithKeys("j", "down"),
			key.WithHelp("j/↓", "move down"),
		),
		Left: key.NewBinding(
			key.WithKeys("h", "left"),
			key.WithHelp("h/←", "move left"),
		),
		Right: key.NewBinding(
			key.WithKeys("l", "right"),
			key.WithHelp("l/→", "move right"),
		),
	}
}

// OutputPanel represents the query result output panel
type OutputPanel struct {
	viewport    viewport.Model
	focused     bool
	width       int
	height      int
	keyMap      OutputPanelKeyMap
	lastResult  *db.QueryResult
	selectedRow int
	selectedCol int
	rowOffset   int
	colOffset   int
	cellWidth   int
}

// NewOutputPanel creates a new output panel
func NewOutputPanel() *OutputPanel {
	vp := viewport.New(80, 20)
	vp.SetContent("No results to display")

	return &OutputPanel{
		viewport:    vp,
		keyMap:      DefaultOutputPanelKeyMap(),
		lastResult:  nil,
		selectedRow: 0,
		selectedCol: 0,
		rowOffset:   0,
		colOffset:   0,
		cellWidth:   15, // Default cell width
	}
}

// Init initializes the output panel
func (o *OutputPanel) Init() tea.Cmd {
	return nil
}

// Update handles messages and updates the output panel
func (o *OutputPanel) Update(msg tea.Msg) (*OutputPanel, tea.Cmd) {
	var cmds []tea.Cmd

	switch msg := msg.(type) {
	case tea.KeyMsg:
		if !o.focused || o.lastResult == nil {
			break
		}

		// Handle navigation
		switch {
		case key.Matches(msg, o.keyMap.Up):
			if o.selectedRow > 0 {
				o.selectedRow--
			}

		case key.Matches(msg, o.keyMap.Down):
			if o.selectedRow < len(o.lastResult.Rows)-1 {
				o.selectedRow++
			}

		case key.Matches(msg, o.keyMap.Left):
			if o.selectedCol > 0 {
				o.selectedCol--
			}

		case key.Matches(msg, o.keyMap.Right):
			if o.selectedCol < len(o.lastResult.Columns)-1 {
				o.selectedCol++
			}
		}

		// Ensure the selected cell is visible
		o.ensureSelectionVisible()

		// Update content
		o.viewport.SetContent(o.renderTable())

	case QueryExecutedMsg:
		// Update with new query results
		o.lastResult = msg.Result
		o.selectedRow = 0
		o.selectedCol = 0
		o.rowOffset = 0
		o.colOffset = 0

		// Update content
		content := o.renderTable()
		o.viewport.SetContent(content)
		o.viewport.GotoTop()
	}

	// Update viewport
	var cmd tea.Cmd
	o.viewport, cmd = o.viewport.Update(msg)
	cmds = append(cmds, cmd)

	return o, tea.Batch(cmds...)
}

// View renders the output panel
func (o *OutputPanel) View() string {
	return lipgloss.NewStyle().
		BorderStyle(lipgloss.NormalBorder()).
		BorderForeground(o.getBorderColor()).
		Width(o.width).
		Height(o.height).
		Render(o.viewport.View())
}

// SetSize sets the panel dimensions
func (o *OutputPanel) SetSize(width, height int) {
	o.width = width
	o.height = height

	// Adjust viewport size to fit within the panel
	o.viewport.Width = width - 2 // Account for borders
	o.viewport.Height = height - 2

	// If we have results, update the table rendering
	if o.lastResult != nil {
		o.viewport.SetContent(o.renderTable())
	}
}

// SetFocused sets whether the panel is focused
func (o *OutputPanel) SetFocused(focused bool) {
	o.focused = focused
}

// getBorderColor returns the border color based on focus
func (o *OutputPanel) getBorderColor() lipgloss.Color {
	if o.focused {
		return lipgloss.Color("12") // Bright blue for focused
	}
	return lipgloss.Color("8") // Gray for unfocused
}

// renderTable renders the query result as a table
func (o *OutputPanel) renderTable() string {
	if o.lastResult == nil || len(o.lastResult.Columns) == 0 {
		return "No results to display"
	}

	// Calculate visible columns based on viewport width and cell width
	visibleCols := (o.viewport.Width - 2) / (o.cellWidth + 1) // +1 for separator
	if visibleCols < 1 {
		visibleCols = 1
	}

	// Calculate visible rows based on viewport height
	visibleRows := o.viewport.Height - 3 // 1 for header, 1 for separator, 1 for message
	if visibleRows < 1 {
		visibleRows = 1
	}

	// Adjust offsets if necessary to keep selection visible
	o.ensureSelectionVisible()

	// Build the table
	var sb strings.Builder

	// Add column headers
	headers := make([]string, 0)
	for i := o.colOffset; i < len(o.lastResult.Columns) && i-o.colOffset < visibleCols; i++ {
		colName := o.lastResult.Columns[i]
		if len(colName) > o.cellWidth {
			colName = colName[:o.cellWidth-3] + "..."
		}

		// Highlight selected column
		if i == o.selectedCol && o.focused {
			colName = lipgloss.NewStyle().
				Bold(true).
				Foreground(lipgloss.Color("15")). // White
				Background(lipgloss.Color("12")). // Blue
				Width(o.cellWidth).
				Align(lipgloss.Center).
				Render(colName)
		} else {
			colName = lipgloss.NewStyle().
				Bold(true).
				Foreground(lipgloss.Color("14")). // Yellow
				Width(o.cellWidth).
				Align(lipgloss.Center).
				Render(colName)
		}

		headers = append(headers, colName)
	}
	sb.WriteString(strings.Join(headers, "│") + "\n")

	// Add header separator
	sb.WriteString(strings.Repeat("─", o.viewport.Width) + "\n")

	// Add rows
	for i := o.rowOffset; i < len(o.lastResult.Rows) && i-o.rowOffset < visibleRows; i++ {
		row := o.lastResult.Rows[i]
		cells := make([]string, 0)

		for j := o.colOffset; j < len(o.lastResult.Columns) && j-o.colOffset < visibleCols; j++ {
			// Get cell value and convert to string
			var cellValue string
			if j < len(row) {
				cellValue = fmt.Sprintf("%v", row[j])
			} else {
				cellValue = ""
			}

			// Truncate if too long
			if len(cellValue) > o.cellWidth {
				cellValue = cellValue[:o.cellWidth-3] + "..."
			}

			// Highlight selected cell
			if i == o.selectedRow && j == o.selectedCol && o.focused {
				cellValue = lipgloss.NewStyle().
					Foreground(lipgloss.Color("15")). // White
					Background(lipgloss.Color("12")). // Blue
					Width(o.cellWidth).
					Align(lipgloss.Left).
					Render(cellValue)
			} else if i == o.selectedRow && o.focused {
				// Highlight selected row
				cellValue = lipgloss.NewStyle().
					Foreground(lipgloss.Color("15")). // White
					Background(lipgloss.Color("8")).  // Gray
					Width(o.cellWidth).
					Align(lipgloss.Left).
					Render(cellValue)
			} else {
				cellValue = lipgloss.NewStyle().
					Width(o.cellWidth).
					Align(lipgloss.Left).
					Render(cellValue)
			}

			cells = append(cells, cellValue)
		}

		sb.WriteString(strings.Join(cells, "│") + "\n")
	}

	// Add result message
	sb.WriteString("\n" + o.lastResult.Message)

	return sb.String()
}

// ensureSelectionVisible adjusts the offsets to ensure the selected cell is visible
func (o *OutputPanel) ensureSelectionVisible() {
	// Calculate visible columns based on viewport width and cell width
	visibleCols := (o.viewport.Width - 2) / (o.cellWidth + 1) // +1 for separator
	if visibleCols < 1 {
		visibleCols = 1
	}

	// Calculate visible rows based on viewport height
	visibleRows := o.viewport.Height - 3 // 1 for header, 1 for separator, 1 for message
	if visibleRows < 1 {
		visibleRows = 1
	}

	// Adjust column offset if selected column is outside visible area
	if o.selectedCol < o.colOffset {
		o.colOffset = o.selectedCol
	} else if o.selectedCol >= o.colOffset+visibleCols {
		o.colOffset = o.selectedCol - visibleCols + 1
	}

	// Adjust row offset if selected row is outside visible area
	if o.selectedRow < o.rowOffset {
		o.rowOffset = o.selectedRow
	} else if o.selectedRow >= o.rowOffset+visibleRows {
		o.rowOffset = o.selectedRow - visibleRows + 1
	}
}
