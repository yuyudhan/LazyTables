// Database-adaptive tables pane component

use crate::{app::AppState, database::objects::DatabaseObjectList};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

/// Render the tables pane with database-adaptive features
pub fn render_tables_pane(
    frame: &mut Frame,
    area: Rect,
    state: &mut AppState,
    theme: &crate::ui::theme::Theme,
) {
    let is_focused = state.ui.focused_pane == crate::app::FocusedPane::Tables;
    let border_style = if is_focused {
        Style::default().fg(theme.get_color("active_border"))
    } else {
        Style::default().fg(theme.get_color("border"))
    };

    // Check if there's an active connection
    let has_active_connection = state
        .db
        .connections
        .connections
        .iter()
        .any(|conn| conn.is_connected());

    // Get items using the new unified selection system
    let items: Vec<ListItem> = if !has_active_connection {
        get_no_connection_message(is_focused)
    } else if state.ui.selectable_table_items.is_empty() {
        get_no_tables_message(state)
    } else {
        // Use filtered items if search is active, otherwise use all items
        let display_items = state.ui.get_display_table_items();
        get_selectable_items_list(display_items, is_focused, &state.ui)
    };

    // Build adaptive title with object counts and schema info
    let title = get_adaptive_title(&state.db.database_objects, &state.db, &state.ui);

    let tables = List::new(items)
        .block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(border_style),
        )
        .highlight_style(
            Style::default()
                .bg(theme.get_color("selection_bg"))
                .add_modifier(Modifier::BOLD),
        );

    frame.render_stateful_widget(tables, area, &mut state.ui.tables_list_state);
}

/// Get message when no database is connected
fn get_no_connection_message(_is_focused: bool) -> Vec<ListItem<'static>> {
    vec![
        ListItem::new(Line::from(vec![Span::styled(
            "Choose a connection",
            Style::default().fg(Color::Gray),
        )])),
        ListItem::new(""),
        ListItem::new(Line::from(vec![Span::styled(
            "from the Connections pane",
            Style::default().fg(Color::Gray),
        )])),
        ListItem::new(Line::from(vec![Span::styled(
            "to view tables and views",
            Style::default().fg(Color::Gray),
        )])),
    ]
}

/// Get message when database is connected but no tables exist
fn get_no_tables_message(state: &AppState) -> Vec<ListItem<'static>> {
    let message: &'static str = if let Some(connection) = state
        .db
        .connections
        .connections
        .get(state.ui.selected_connection)
    {
        match &connection.status {
            crate::database::ConnectionStatus::Connected => "No tables in database",
            crate::database::ConnectionStatus::Connecting => "Connecting to database...",
            crate::database::ConnectionStatus::Failed(_error) => {
                "Connection failed (see status bar)"
            }
            crate::database::ConnectionStatus::Disconnected => "Not connected",
        }
    } else {
        "No connection selected"
    };

    vec![ListItem::new(Line::from(vec![Span::styled(
        message,
        Style::default().fg(if message.contains("failed") {
            Color::Red
        } else {
            Color::Yellow
        }),
    )]))]
}

/// Get list items from selectable table items
fn get_selectable_items_list(
    selectable_items: &[crate::state::ui::SelectableTableItem],
    _is_focused: bool,
    ui_state: &crate::state::ui::UIState,
) -> Vec<ListItem<'static>> {
    let mut items = Vec::new();

    for item in selectable_items {
        if item.display_name.is_empty() {
            // Empty line separator
            items.push(ListItem::new(""));
        } else if item.is_selectable {
            // Selectable table/view item
            items.push(ListItem::new(Line::from(vec![Span::styled(
                item.display_name.clone(),
                Style::default().fg(Color::White),
            )])));
        } else {
            // Group header
            items.push(ListItem::new(Line::from(vec![Span::styled(
                item.display_name.clone(),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )])));
        }
    }

    // Show active search query if in search mode
    if ui_state.tables_search_active {
        items.push(ListItem::new(""));
        items.push(ListItem::new(Line::from(vec![
            Span::styled(
                "Search: ",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("{}_", ui_state.tables_search_query),
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::UNDERLINED),
            ),
        ])));
    }

    items
}


/// Get adaptive title based on database objects and connection info
fn get_adaptive_title(
    db_objects: &Option<DatabaseObjectList>,
    db_state: &crate::state::DatabaseState,
    ui_state: &crate::state::ui::UIState,
) -> String {
    if let Some(ref objects) = db_objects {
        let mut title_parts = Vec::new();

        // Add schema info for databases that support multiple schemas
        if db_state.schemas.len() > 1 {
            let schema = db_state.selected_schema.as_deref().unwrap_or("all");
            title_parts.push(format!("Schema: {}", schema));
        }

        // Add object counts
        let mut counts = Vec::new();
        if !objects.tables.is_empty() {
            counts.push(format!("{} tables", objects.tables.len()));
        }
        if !objects.views.is_empty() {
            counts.push(format!("{} views", objects.views.len()));
        }
        if !objects.materialized_views.is_empty() {
            counts.push(format!("{} mat. views", objects.materialized_views.len()));
        }
        if !objects.foreign_tables.is_empty() {
            counts.push(format!("{} foreign", objects.foreign_tables.len()));
        }

        if !counts.is_empty() {
            title_parts.push(counts.join(", "));
        }

        let base_title = if !title_parts.is_empty() {
            format!(" Tables/Views ({}) ", title_parts.join(" | "))
        } else {
            " Tables/Views ".to_string()
        };

        // Add search indicator if search is active
        if ui_state.tables_search_active {
            let filter_count = ui_state.filtered_table_items.len();
            format!(
                "{} [Search: {} result{}]",
                base_title.trim(),
                filter_count,
                if filter_count == 1 { "" } else { "s" }
            )
        } else {
            base_title
        }
    } else {
        " Tables/Views ".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        database::objects::{DatabaseObject, DatabaseObjectType},
        state::ui::SelectableTableItem,
    };

    #[test]
    fn test_selectable_table_item_creation() {
        let item = SelectableTableItem::new_selectable(
            "  📋 users".to_string(),
            "users".to_string(),
            Some("public".to_string()),
            DatabaseObjectType::Table,
            0,
        );

        assert_eq!(item.display_name, "  📋 users");
        assert_eq!(item.object_name, "users");
        assert_eq!(item.schema, Some("public".to_string()));
        assert!(item.is_selectable);
        assert_eq!(item.qualified_name(), "public.users");
    }

    #[test]
    fn test_selectable_table_item_header() {
        let item = SelectableTableItem::new_header("▼ Tables".to_string(), 0);

        assert_eq!(item.display_name, "▼ Tables");
        assert!(!item.is_selectable);
    }

    #[test]
    fn test_get_selectable_items_list_empty() {
        let ui_state = crate::state::ui::UIState::new();
        let items = get_selectable_items_list(&[], false, &ui_state);
        assert!(items.is_empty());
    }

    #[test]
    fn test_get_selectable_items_list_with_items() {
        let selectable_items = vec![
            SelectableTableItem::new_header("▼ Tables".to_string(), 0),
            SelectableTableItem::new_selectable(
                "  📋 users".to_string(),
                "users".to_string(),
                None,
                DatabaseObjectType::Table,
                1,
            ),
            SelectableTableItem::new_selectable(
                "  📋 posts".to_string(),
                "posts".to_string(),
                None,
                DatabaseObjectType::Table,
                2,
            ),
        ];

        let ui_state = crate::state::ui::UIState::new();
        let items = get_selectable_items_list(&selectable_items, false, &ui_state);

        // Should have 3 items (header + 2 tables) without navigation help
        assert_eq!(items.len(), 3);
    }

    #[test]
    fn test_get_selectable_items_list_with_focus() {
        let selectable_items = vec![SelectableTableItem::new_selectable(
            "  📋 users".to_string(),
            "users".to_string(),
            None,
            DatabaseObjectType::Table,
            0,
        )];

        let ui_state = crate::state::ui::UIState::new();
        let items = get_selectable_items_list(&selectable_items, true, &ui_state);

        // Should have just the table item (no help text is shown in pane anymore)
        assert_eq!(items.len(), 1);
    }

    #[test]
    fn test_get_adaptive_title_with_objects() {
        let objects = DatabaseObjectList {
            tables: vec![DatabaseObject {
                name: "users".to_string(),
                schema: None,
                object_type: DatabaseObjectType::Table,
                row_count: None,
                size_bytes: None,
                comment: None,
            }],
            views: vec![],
            materialized_views: vec![],
            foreign_tables: vec![],
            total_count: 1,
            error: None,
        };

        let db_state = crate::state::DatabaseState::new();
        let ui_state = crate::state::ui::UIState::new();
        let title = get_adaptive_title(&Some(objects), &db_state, &ui_state);

        assert!(title.contains("1 tables"));
    }

    #[test]
    fn test_get_adaptive_title_empty() {
        let db_state = crate::state::DatabaseState::new();
        let ui_state = crate::state::ui::UIState::new();
        let title = get_adaptive_title(&None, &db_state, &ui_state);

        assert_eq!(title, " Tables/Views ");
    }
}
