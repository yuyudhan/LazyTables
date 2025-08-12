// FilePath: internal/ui/ui.go

package ui

import (
	"time"

	"github.com/charmbracelet/bubbles/key"
	tea "github.com/charmbracelet/bubbletea"
	"github.com/charmbracelet/lipgloss"
	"github.com/yuyudhan/LazyTables/internal/ui/components"
	"github.com/yuyudhan/LazyTables/internal/ui/layout"
	"github.com/yuyudhan/LazyTables/pkg/logger"
)

// UI represents the main user interface
type UI struct {
	program        *tea.Program
	layout         *layout.Layout
	keyMap         KeyMap
	focusedPanel   components.PanelType
	notifications  *components.NotificationManager
	lastWindowSize tea.WindowSizeMsg
}

// KeyMap defines the keybindings for the UI
type KeyMap struct {
	FocusConnections  key.Binding
	ToggleConnections key.Binding
	FocusDatabases    key.Binding
	ToggleDatabases   key.Binding
	FocusTables       key.Binding
	ToggleTables      key.Binding
	FocusQuery        key.Binding
	ToggleQuery       key.Binding
	FocusOutput       key.Binding
	ToggleOutput      key.Binding
	Quit              key.Binding
}

// DefaultKeyMap returns the default keybindings
func DefaultKeyMap() KeyMap {
	return KeyMap{
		FocusConnections: key.NewBinding(
			key.WithKeys("c"),
			key.WithHelp("c", "focus connections"),
		),
		ToggleConnections: key.NewBinding(
			key.WithKeys("C"),
			key.WithHelp("C", "toggle connections"),
		),
		FocusDatabases: key.NewBinding(
			key.WithKeys("d"),
			key.WithHelp("d", "focus databases"),
		),
		ToggleDatabases: key.NewBinding(
			key.WithKeys("D"),
			key.WithHelp("D", "toggle databases"),
		),
		FocusTables: key.NewBinding(
			key.WithKeys("t"),
			key.WithHelp("t", "focus tables"),
		),
		ToggleTables: key.NewBinding(
			key.WithKeys("T"),
			key.WithHelp("T", "toggle tables"),
		),
		FocusQuery: key.NewBinding(
			key.WithKeys("q"),
			key.WithHelp("q", "focus query"),
		),
		ToggleQuery: key.NewBinding(
			key.WithKeys("Q"),
			key.WithHelp("Q", "toggle query"),
		),
		FocusOutput: key.NewBinding(
			key.WithKeys("o"),
			key.WithHelp("o", "focus output"),
		),
		ToggleOutput: key.NewBinding(
			key.WithKeys("O"),
			key.WithHelp("O", "toggle output"),
		),
		Quit: key.NewBinding(
			key.WithKeys("ctrl+c", "esc"),
			key.WithHelp("ctrl+c/esc", "quit"),
		),
	}
}

// NewUI creates a new UI instance
func NewUI() *UI {
	// Initialize styles
	lipgloss.SetColorProfile(lipgloss.ColorProfile256)

	ui := &UI{
		keyMap:        DefaultKeyMap(),
		focusedPanel:  components.PanelConnections,
		notifications: components.NewNotificationManager(3 * time.Second),
	}

	// Initialize layout
	ui.layout = layout.NewLayout()

	logger.Info("UI initialized")
	return ui
}

// Start launches the UI
func (ui *UI) Start() error {
	ui.program = tea.NewProgram(ui, tea.WithAltScreen())

	// Run the program
	_, err := ui.program.Run()
	return err
}

// Init implements tea.Model
func (ui *UI) Init() tea.Cmd {
	// Initialize all components
	cmds := []tea.Cmd{
		ui.layout.Init(),
		ui.notifications.Init(),
	}

	// Set initial focus
	ui.SetFocus(components.PanelConnections)

	return tea.Batch(cmds...)
}

// Update implements tea.Model
func (ui *UI) Update(msg tea.Msg) (tea.Model, tea.Cmd) {
	var cmds []tea.Cmd

	switch msg := msg.(type) {
	case tea.KeyMsg:
		// Global keybindings
		switch {
		case key.Matches(msg, ui.keyMap.Quit):
			return ui, tea.Quit

		case key.Matches(msg, ui.keyMap.FocusConnections):
			ui.SetFocus(components.PanelConnections)
			return ui, nil

		case key.Matches(msg, ui.keyMap.ToggleConnections):
			ui.layout.ToggleConnections()
			return ui, nil

		case key.Matches(msg, ui.keyMap.FocusDatabases):
			ui.SetFocus(components.PanelDatabases)
			return ui, nil

		case key.Matches(msg, ui.keyMap.ToggleDatabases):
			ui.layout.ToggleDatabases()
			return ui, nil

		case key.Matches(msg, ui.keyMap.FocusTables):
			ui.SetFocus(components.PanelTables)
			return ui, nil

		case key.Matches(msg, ui.keyMap.ToggleTables):
			ui.layout.ToggleTables()
			return ui, nil

		case key.Matches(msg, ui.keyMap.FocusQuery):
			ui.SetFocus(components.PanelQuery)
			return ui, nil

		case key.Matches(msg, ui.keyMap.ToggleQuery):
			ui.layout.ToggleQuery()
			return ui, nil

		case key.Matches(msg, ui.keyMap.FocusOutput):
			ui.SetFocus(components.PanelOutput)
			return ui, nil

		case key.Matches(msg, ui.keyMap.ToggleOutput):
			ui.layout.ToggleOutput()
			return ui, nil
		}

	case tea.WindowSizeMsg:
		// Store window size for potential use elsewhere
		ui.lastWindowSize = msg

		// Update layout with new window size
		layoutCmd := ui.layout.UpdateSize(msg.Width, msg.Height)
		cmds = append(cmds, layoutCmd)

		// Update notification manager with new window size
		notifCmd := ui.notifications.UpdateSize(msg.Width, msg.Height)
		cmds = append(cmds, notifCmd)

	case components.NotificationMsg:
		// Handle notifications
		cmd := ui.notifications.Add(msg.Type, msg.Title, msg.Content)
		cmds = append(cmds, cmd)
	}

	// Update the layout
	layout, cmd := ui.layout.Update(msg)
	ui.layout = layout.(*layout.Layout)
	cmds = append(cmds, cmd)

	// Update the notification manager
	notifications, cmd := ui.notifications.Update(msg)
	ui.notifications = notifications.(*components.NotificationManager)
	cmds = append(cmds, cmd)

	return ui, tea.Batch(cmds...)
}

// View implements tea.Model
func (ui *UI) View() string {
	// Combine layout and notifications
	layoutView := ui.layout.View()
	notificationsView := ui.notifications.View()

	// Final view is the layout with notifications overlaid
	return layoutView + notificationsView
}

// SetFocus sets focus on a specific panel
func (ui *UI) SetFocus(panel components.PanelType) {
	ui.focusedPanel = panel
	ui.layout.SetFocus(panel)
}

// ShowNotification displays a notification
func (ui *UI) ShowNotification(notifType components.NotificationType, title, content string) {
	if ui.program != nil {
		ui.program.Send(components.NotificationMsg{
			Type:    notifType,
			Title:   title,
			Content: content,
		})
	}
}
