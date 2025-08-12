// FilePath: internal/ui/components/input_dialog.go

package components

import (
	"strings"

	"github.com/charmbracelet/bubbles/key"
	"github.com/charmbracelet/bubbles/textinput"
	tea "github.com/charmbracelet/bubbletea"
	"github.com/charmbracelet/lipgloss"
)

// DialogField represents a field in the input dialog
type DialogField struct {
	Label       string
	Placeholder string
	Value       string
	IsPassword  bool
}

// DialogResult contains the result of an input dialog
type DialogResult struct {
	Confirmed bool
	Fields    map[string]string
}

// InputDialogMsg is sent when a dialog is completed
type InputDialogMsg struct {
	ID     string
	Result DialogResult
}

// InputDialogKeyMap defines the keybindings for the input dialog
type InputDialogKeyMap struct {
	NextField     key.Binding
	PrevField     key.Binding
	Confirm       key.Binding
	Cancel        key.Binding
	MoveSelUpDown key.Binding
}

// DefaultInputDialogKeyMap returns the default keybindings
func DefaultInputDialogKeyMap() InputDialogKeyMap {
	return InputDialogKeyMap{
		NextField: key.NewBinding(
			key.WithKeys("tab"),
			key.WithHelp("tab", "next field"),
		),
		PrevField: key.NewBinding(
			key.WithKeys("shift+tab"),
			key.WithHelp("shift+tab", "previous field"),
		),
		Confirm: key.NewBinding(
			key.WithKeys("enter"),
			key.WithHelp("enter", "confirm"),
		),
		Cancel: key.NewBinding(
			key.WithKeys("esc"),
			key.WithHelp("esc", "cancel"),
		),
		MoveSelUpDown: key.NewBinding(
			key.WithKeys("up", "down", "k", "j"),
		),
	}
}

// InputDialog represents a modal dialog for input
type InputDialog struct {
	id          string
	title       string
	width       int
	height      int
	fields      []DialogField
	inputs      []textinput.Model
	activeInput int
	keyMap      InputDialogKeyMap
	onResult    func(result DialogResult)
}

// NewInputDialog creates a new input dialog
func NewInputDialog(id, title string, fields []DialogField, onResult func(result DialogResult)) *InputDialog {
	inputs := make([]textinput.Model, len(fields))

	for i, field := range fields {
		ti := textinput.New()
		ti.Placeholder = field.Placeholder
		ti.PromptStyle = lipgloss.NewStyle().Foreground(lipgloss.Color("12"))
		ti.TextStyle = lipgloss.NewStyle().Foreground(lipgloss.Color("15"))

		if i == 0 {
			ti.Focus()
		}

		if field.IsPassword {
			ti.EchoMode = textinput.EchoPassword
		}

		if field.Value != "" {
			ti.SetValue(field.Value)
		}

		inputs[i] = ti
	}

	return &InputDialog{
		id:          id,
		title:       title,
		fields:      fields,
		inputs:      inputs,
		activeInput: 0,
		keyMap:      DefaultInputDialogKeyMap(),
		onResult:    onResult,
		width:       50, // Default width
		height:      0,  // Will be calculated based on content
	}
}

// Init initializes the input dialog
func (d *InputDialog) Init() tea.Cmd {
	return textinput.Blink
}

// Update handles messages and updates the input dialog
func (d *InputDialog) Update(msg tea.Msg) (tea.Model, tea.Cmd) {
	switch msg := msg.(type) {
	case tea.KeyMsg:
		// Handle keys
		switch {
		case key.Matches(msg, d.keyMap.NextField):
			// Move to next field
			d.activeInput = (d.activeInput + 1) % len(d.inputs)
			cmds := make([]tea.Cmd, len(d.inputs))

			for i := range d.inputs {
				if i == d.activeInput {
					cmds[i] = d.inputs[i].Focus()
				} else {
					d.inputs[i].Blur()
				}
			}

			return d, tea.Batch(cmds...)

		case key.Matches(msg, d.keyMap.PrevField):
			// Move to previous field
			d.activeInput = (d.activeInput - 1 + len(d.inputs)) % len(d.inputs)
			cmds := make([]tea.Cmd, len(d.inputs))

			for i := range d.inputs {
				if i == d.activeInput {
					cmds[i] = d.inputs[i].Focus()
				} else {
					d.inputs[i].Blur()
				}
			}

			return d, tea.Batch(cmds...)

		case key.Matches(msg, d.keyMap.Confirm):
			// Create result map
			result := DialogResult{
				Confirmed: true,
				Fields:    make(map[string]string),
			}

			for i, field := range d.fields {
				result.Fields[field.Label] = d.inputs[i].Value()
			}

			// Call result callback
			if d.onResult != nil {
				d.onResult(result)
			}

			// Return message
			return d, func() tea.Msg {
				return InputDialogMsg{
					ID:     d.id,
					Result: result,
				}
			}

		case key.Matches(msg, d.keyMap.Cancel):
			// Cancel dialog
			result := DialogResult{
				Confirmed: false,
				Fields:    make(map[string]string),
			}

			// Call result callback
			if d.onResult != nil {
				d.onResult(result)
			}

			// Return message
			return d, func() tea.Msg {
				return InputDialogMsg{
					ID:     d.id,
					Result: result,
				}
			}
		}
	}

	// Handle input updates
	cmd := d.updateInputs(msg)
	return d, cmd
}

// View renders the input dialog
func (d *InputDialog) View() string {
	// Dialog style
	dialogStyle := lipgloss.NewStyle().
		Border(lipgloss.RoundedBorder()).
		BorderForeground(lipgloss.Color("12")).
		Padding(1, 3)

	// Title style
	titleStyle := lipgloss.NewStyle().
		Bold(true).
		Foreground(lipgloss.Color("15")).
		Background(lipgloss.Color("12")).
		Padding(0, 1)

	// Build dialog content
	var sb strings.Builder

	// Add title
	sb.WriteString(titleStyle.Render(d.title) + "\n\n")

	// Add fields
	for i, field := range d.fields {
		sb.WriteString(field.Label + ":\n")
		sb.WriteString(d.inputs[i].View() + "\n\n")
	}

	// Add buttons
	buttonStyle := lipgloss.NewStyle().
		Foreground(lipgloss.Color("15")).
		Background(lipgloss.Color("12")).
		Padding(0, 3)

	cancelButtonStyle := lipgloss.NewStyle().
		Foreground(lipgloss.Color("15")).
		Background(lipgloss.Color("8")).
		Padding(0, 3)

	buttons := lipgloss.JoinHorizontal(
		lipgloss.Center,
		buttonStyle.Render("OK [Enter]"),
		"  ",
		cancelButtonStyle.Render("Cancel [Esc]"),
	)
	sb.WriteString(lipgloss.NewStyle().
		Align(lipgloss.Center).
		Width(d.width - 8). // Adjust for padding and border
		Render(buttons))

	// Render dialog
	return dialogStyle.
		Width(d.width).
		Render(sb.String())
}

// updateInputs updates all text inputs
func (d *InputDialog) updateInputs(msg tea.Msg) tea.Cmd {
	cmds := make([]tea.Cmd, len(d.inputs))

	// Only update the active input
	var cmd tea.Cmd
	d.inputs[d.activeInput], cmd = d.inputs[d.activeInput].Update(msg)
	cmds[d.activeInput] = cmd

	return tea.Batch(cmds...)
}

// SetSize sets the dimensions of the dialog
func (d *InputDialog) SetSize(width, height int) {
	d.width = width
	d.height = height
}

// GetSize returns the dialog's dimensions
func (d *InputDialog) GetSize() (int, int) {
	return d.width, d.height
}
