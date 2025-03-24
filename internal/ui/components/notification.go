// FilePath: internal/ui/components/notification.go

package components

import (
	"strings"
	"time"

	tea "github.com/charmbracelet/bubbletea"
	"github.com/charmbracelet/lipgloss"
)

// NotificationType defines the type of notification
type NotificationType int

const (
	NotificationInfo NotificationType = iota
	NotificationError
	NotificationWarning
	NotificationSuccess
)

// Notification represents a single notification
type Notification struct {
	ID        int
	Type      NotificationType
	Title     string
	Content   string
	CreatedAt time.Time
	ExpiresAt time.Time
}

// NotificationMsg is sent when a notification is created
type NotificationMsg struct {
	Type    NotificationType
	Title   string
	Content string
}

// NotificationExpiredMsg is sent when a notification expires
type NotificationExpiredMsg struct {
	ID int
}

// NotificationManager manages the display of notifications
type NotificationManager struct {
	notifications []Notification
	width         int
	height        int
	nextID        int
	duration      time.Duration
}

// NewNotificationManager creates a new notification manager
func NewNotificationManager(duration time.Duration) *NotificationManager {
	return &NotificationManager{
		notifications: []Notification{},
		nextID:        1,
		duration:      duration,
	}
}

// Init initializes the notification manager
func (n *NotificationManager) Init() tea.Cmd {
	return nil
}

// Update handles messages and updates the notification manager
func (n *NotificationManager) Update(msg tea.Msg) (tea.Model, tea.Cmd) {
	var cmds []tea.Cmd

	switch msg := msg.(type) {
	case NotificationExpiredMsg:
		// Remove expired notification
		for i, notification := range n.notifications {
			if notification.ID == msg.ID {
				n.notifications = append(n.notifications[:i], n.notifications[i+1:]...)
				break
			}
		}

	case NotificationMsg:
		// Create new notification
		notification := Notification{
			ID:        n.nextID,
			Type:      msg.Type,
			Title:     msg.Title,
			Content:   msg.Content,
			CreatedAt: time.Now(),
			ExpiresAt: time.Now().Add(n.duration),
		}
		n.nextID++

		n.notifications = append(n.notifications, notification)

		// Schedule expiration
		cmds = append(cmds, n.expireCmd(notification.ID, n.duration))
	}

	return n, tea.Batch(cmds...)
}

// View renders the notification manager
func (n *NotificationManager) View() string {
	if len(n.notifications) == 0 {
		return ""
	}

	// Create notification views
	var notificationViews []string

	// Define styles for different notification types
	infoStyle := lipgloss.NewStyle().
		BorderStyle(lipgloss.RoundedBorder()).
		BorderForeground(lipgloss.Color("12")). // Blue
		Padding(0, 1)

	errorStyle := lipgloss.NewStyle().
		BorderStyle(lipgloss.RoundedBorder()).
		BorderForeground(lipgloss.Color("9")). // Red
		Padding(0, 1)

	warningStyle := lipgloss.NewStyle().
		BorderStyle(lipgloss.RoundedBorder()).
		BorderForeground(lipgloss.Color("11")). // Yellow
		Padding(0, 1)

	successStyle := lipgloss.NewStyle().
		BorderStyle(lipgloss.RoundedBorder()).
		BorderForeground(lipgloss.Color("10")). // Green
		Padding(0, 1)

	for _, notification := range n.notifications {
		// Choose style based on notification type
		var style lipgloss.Style
		var typeStr string

		switch notification.Type {
		case NotificationInfo:
			style = infoStyle
			typeStr = "INFO"
		case NotificationError:
			style = errorStyle
			typeStr = "ERROR"
		case NotificationWarning:
			style = warningStyle
			typeStr = "WARNING"
		case NotificationSuccess:
			style = successStyle
			typeStr = "SUCCESS"
		}

		// Calculate maximum content width (adjust based on screen width)
		maxWidth := n.width / 4

		// Create title with type indicator
		title := lipgloss.NewStyle().
			Bold(true).
			Render(typeStr + ": " + notification.Title)

		// Render content with word wrapping
		content := n.wrapText(notification.Content, maxWidth)

		// Combine title and content
		notificationContent := lipgloss.JoinVertical(
			lipgloss.Left,
			title,
			content,
		)

		// Add to list of notification views
		notificationViews = append(notificationViews, style.Render(notificationContent))
	}

	// Stack the notifications
	stackedNotifications := lipgloss.JoinVertical(
		lipgloss.Left,
		notificationViews...,
	)

	// Position in the top right
	return lipgloss.NewStyle().
		Width(n.width).
		Height(n.height).
		Align(lipgloss.Right, lipgloss.Top).
		Render(stackedNotifications)
}

// Add adds a new notification
func (n *NotificationManager) Add(notifType NotificationType, title, content string) tea.Cmd {
	return func() tea.Msg {
		return NotificationMsg{
			Type:    notifType,
			Title:   title,
			Content: content,
		}
	}
}

// UpdateSize updates the dimensions of the notification manager
func (n *NotificationManager) UpdateSize(width, height int) tea.Cmd {
	n.width = width
	n.height = height
	return nil
}

// expireCmd returns a command that expires a notification after a duration
func (n *NotificationManager) expireCmd(id int, duration time.Duration) tea.Cmd {
	return tea.Tick(duration, func(time.Time) tea.Msg {
		return NotificationExpiredMsg{ID: id}
	})
}

// wrapText wraps text to a maximum width
func (n *NotificationManager) wrapText(text string, maxWidth int) string {
	if maxWidth <= 0 {
		return text
	}

	words := strings.Fields(text)
	if len(words) == 0 {
		return ""
	}

	var result []string
	line := words[0]

	for _, word := range words[1:] {
		if len(line)+len(word)+1 <= maxWidth {
			line += " " + word
		} else {
			result = append(result, line)
			line = word
		}
	}

	result = append(result, line)
	return strings.Join(result, "\n")
}
