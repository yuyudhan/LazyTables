// Database-adaptive tables pane component

use crate::{
    app::AppState,
    database::{objects::DatabaseObjectList, DatabaseType},
};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

/// Render the tables pane with database-adaptive features
pub fn render_tables_pane(frame: &mut Frame, area: Rect, state: &mut AppState, theme: &crate::ui::theme::Theme) {
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

    // Determine if we need selection
    let should_show_selection = !state.db.tables.is_empty() && has_active_connection;
    let selected_index = if should_show_selection {
        Some(state.ui.selected_table)
    } else {
        None
    };

    // Get items (borrowed immutably)
    let items: Vec<ListItem> = if !has_active_connection {
        get_no_connection_message(is_focused)
    } else if state.db.tables.is_empty() {
        get_no_tables_message(state)
    } else {
        get_database_objects_list(state, is_focused)
    };

    // Build adaptive title with object counts and schema info
    let title = get_adaptive_title(&state.db.database_objects, &state.db);

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

    // Set selection state (mutable borrow after immutable borrows are done)
    if let Some(index) = selected_index {
        state.ui.tables_list_state.select(Some(index));
    }

    frame.render_stateful_widget(tables, area, &mut state.ui.tables_list_state);
}

/// Get message when no database is connected
fn get_no_connection_message(is_focused: bool) -> Vec<ListItem<'static>> {
    let mut items = vec![
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
        ListItem::new(""),
    ];

    if is_focused {
        items.push(ListItem::new(Line::from(vec![
            Span::styled("Press ", Style::default().fg(Color::Gray)),
            Span::styled(
                "Ctrl+h",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" to go to connections", Style::default().fg(Color::Gray)),
        ])));
    } else {
        items.push(ListItem::new(""));
    }

    items
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
            crate::database::ConnectionStatus::Failed(_error) => "Connection failed (see status bar)",
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

/// Get database objects list with database-adaptive features
fn get_database_objects_list(state: &AppState, is_focused: bool) -> Vec<ListItem<'static>> {
    let mut table_items = Vec::new();

    // Add database objects with adaptive icons and grouping
    if let Some(ref db_objects) = state.db.database_objects {
        add_database_objects_by_type(&mut table_items, db_objects, state);
    } else {
        // Fallback to simple table list
        add_simple_table_list(&mut table_items, state);
    }

    // Add navigation help if focused
    if is_focused && !state.db.tables.is_empty() {
        add_navigation_help(&mut table_items);
    }

    table_items
}

/// Add database objects organized by type with adaptive icons
fn add_database_objects_by_type(
    items: &mut Vec<ListItem<'static>>,
    db_objects: &DatabaseObjectList,
    state: &AppState,
) {
    // Get current database type for adaptive behavior
    let db_type = state
        .db
        .connections
        .connections
        .get(state.ui.selected_connection)
        .map(|conn| &conn.database_type);

    // Add tables section
    if !db_objects.tables.is_empty() {
        let is_expanded = state.ui.is_object_group_expanded("Tables");
        add_object_group_header(items, "Tables", db_objects.tables.len(), db_type, is_expanded);

        if is_expanded {
            for table in &db_objects.tables {
                let (icon, color) = get_object_icon_and_color(&table.object_type, db_type);
                let qualified_name = get_qualified_name(table, db_type);

                items.push(ListItem::new(Line::from(vec![
                    Span::styled("  ", Style::default()),
                    Span::styled(format!("{} ", icon), Style::default().fg(color)),
                    Span::styled(qualified_name.clone(), Style::default().fg(Color::White)),
                    get_size_info_span(table.size_bytes),
                ])));
            }
        }
    }

    // Add views section
    if !db_objects.views.is_empty() {
        let is_expanded = state.ui.is_object_group_expanded("Views");
        add_object_group_header(items, "Views", db_objects.views.len(), db_type, is_expanded);

        if is_expanded {
            for view in &db_objects.views {
                let (icon, color) = get_object_icon_and_color(&view.object_type, db_type);
                let qualified_name = get_qualified_name(view, db_type);

                items.push(ListItem::new(Line::from(vec![
                    Span::styled("  ", Style::default()),
                    Span::styled(format!("{} ", icon), Style::default().fg(color)),
                    Span::styled(qualified_name.clone(), Style::default().fg(Color::White)),
                ])));
            }
        }
    }

    // Add materialized views section (PostgreSQL)
    if !db_objects.materialized_views.is_empty() {
        let is_expanded = state.ui.is_object_group_expanded("Materialized Views");
        add_object_group_header(items, "Materialized Views", db_objects.materialized_views.len(), db_type, is_expanded);

        if is_expanded {
            for mv in &db_objects.materialized_views {
                let (icon, color) = get_object_icon_and_color(&mv.object_type, db_type);
                let qualified_name = get_qualified_name(mv, db_type);

                items.push(ListItem::new(Line::from(vec![
                    Span::styled("  ", Style::default()),
                    Span::styled(format!("{} ", icon), Style::default().fg(color)),
                    Span::styled(qualified_name.clone(), Style::default().fg(Color::White)),
                ])));
            }
        }
    }

    // Add foreign tables section (PostgreSQL)
    if !db_objects.foreign_tables.is_empty() {
        let is_expanded = state.ui.is_object_group_expanded("Foreign Tables");
        add_object_group_header(items, "Foreign Tables", db_objects.foreign_tables.len(), db_type, is_expanded);

        if is_expanded {
            for ft in &db_objects.foreign_tables {
                let (icon, color) = get_object_icon_and_color(&ft.object_type, db_type);
                let qualified_name = get_qualified_name(ft, db_type);

                items.push(ListItem::new(Line::from(vec![
                    Span::styled("  ", Style::default()),
                    Span::styled(format!("{} ", icon), Style::default().fg(color)),
                    Span::styled(qualified_name.clone(), Style::default().fg(Color::White)),
                ])));
            }
        }
    }
}

/// Add group header for object types with expansion/collapse support
fn add_object_group_header(
    items: &mut Vec<ListItem<'static>>,
    group_name: &str,
    count: usize,
    db_type: Option<&DatabaseType>,
    is_expanded: bool,
) {
    if !items.is_empty() {
        items.push(ListItem::new(""));
    }

    let group_color = match db_type {
        Some(DatabaseType::PostgreSQL) => Color::Blue,
        Some(DatabaseType::MySQL) => Color::Green,
        Some(DatabaseType::SQLite) => Color::Yellow,
        _ => Color::Cyan,
    };

    let expansion_arrow = if is_expanded { "▼" } else { "▶" };

    items.push(ListItem::new(Line::from(vec![
        Span::styled(
            format!("{} {} ({})", expansion_arrow, group_name, count),
            Style::default()
                .fg(group_color)
                .add_modifier(Modifier::BOLD),
        ),
    ])));
}

/// Get icon and color for database object type with database-specific adaptations
fn get_object_icon_and_color(
    object_type: &crate::database::objects::DatabaseObjectType,
    db_type: Option<&DatabaseType>,
) -> (&'static str, Color) {
    match (object_type, db_type) {
        // PostgreSQL-specific icons
        (crate::database::objects::DatabaseObjectType::Table, Some(DatabaseType::PostgreSQL)) => ("🐘", Color::Blue),
        (crate::database::objects::DatabaseObjectType::View, Some(DatabaseType::PostgreSQL)) => ("👁️", Color::Cyan),
        (crate::database::objects::DatabaseObjectType::MaterializedView, Some(DatabaseType::PostgreSQL)) => ("🔄", Color::Magenta),

        // MySQL-specific icons
        (crate::database::objects::DatabaseObjectType::Table, Some(DatabaseType::MySQL)) => ("🐬", Color::Green),
        (crate::database::objects::DatabaseObjectType::View, Some(DatabaseType::MySQL)) => ("👁️", Color::Green),

        // SQLite-specific icons
        (crate::database::objects::DatabaseObjectType::Table, Some(DatabaseType::SQLite)) => ("💾", Color::Yellow),
        (crate::database::objects::DatabaseObjectType::View, Some(DatabaseType::SQLite)) => ("👁️", Color::Yellow),

        // Default fallback - hardcode the icons to ensure they're 'static
        (crate::database::objects::DatabaseObjectType::Table, _) => ("📋", Color::Blue),
        (crate::database::objects::DatabaseObjectType::View, _) => ("👁️", Color::Green),
        (crate::database::objects::DatabaseObjectType::MaterializedView, _) => ("🔄", Color::Magenta),
        (crate::database::objects::DatabaseObjectType::ForeignTable, _) => ("🔗", Color::Cyan),
        (crate::database::objects::DatabaseObjectType::SystemTable, _) => ("⚙️", Color::DarkGray),
    }
}

/// Get qualified name for database object based on database type
fn get_qualified_name(
    object: &crate::database::objects::DatabaseObject,
    db_type: Option<&DatabaseType>,
) -> String {
    match db_type {
        Some(DatabaseType::PostgreSQL) => {
            // PostgreSQL uses schema.table format
            object.qualified_name()
        }
        Some(DatabaseType::MySQL) => {
            // MySQL uses database.table format
            if let Some(schema) = &object.schema {
                format!("{}.{}", schema, object.name)
            } else {
                object.name.clone()
            }
        }
        Some(DatabaseType::SQLite) => {
            // SQLite typically doesn't use schemas in display
            object.name.clone()
        }
        _ => object.qualified_name(),
    }
}

/// Get size information span if available
fn get_size_info_span(size_bytes: Option<i64>) -> Span<'static> {
    if let Some(size) = size_bytes {
        let formatted_size = format_bytes(size);
        Span::styled(
            format!(" ({})", formatted_size),
            Style::default().fg(Color::DarkGray),
        )
    } else {
        Span::raw("")
    }
}

/// Add simple table list when no database objects are available
fn add_simple_table_list(items: &mut Vec<ListItem<'static>>, state: &AppState) {
    for table in &state.db.tables {
        items.push(ListItem::new(Line::from(vec![
            Span::styled("  📋 ", Style::default().fg(Color::Blue)),
            Span::styled(table.clone(), Style::default().fg(Color::White)),
        ])));
    }
}

/// Add navigation help for focused pane
fn add_navigation_help(items: &mut Vec<ListItem<'static>>) {
    items.push(ListItem::new(""));
    items.push(ListItem::new(Line::from(vec![
        Span::styled("Press ", Style::default().fg(Color::Gray)),
        Span::styled(
            "j/k",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" to navigate tables", Style::default().fg(Color::Gray)),
    ])));
    items.push(ListItem::new(Line::from(vec![
        Span::styled("Press ", Style::default().fg(Color::Gray)),
        Span::styled(
            "Enter",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" to view table data", Style::default().fg(Color::Gray)),
    ])));
    items.push(ListItem::new(Line::from(vec![
        Span::styled("Press ", Style::default().fg(Color::Gray)),
        Span::styled(
            "n",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" to create new table", Style::default().fg(Color::Gray)),
    ])));
}

/// Get adaptive title based on database objects and connection info
fn get_adaptive_title(
    db_objects: &Option<DatabaseObjectList>,
    db_state: &crate::state::DatabaseState,
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

        if !title_parts.is_empty() {
            format!(" Tables/Views ({}) ", title_parts.join(" | "))
        } else {
            " Tables/Views ".to_string()
        }
    } else {
        " Tables/Views ".to_string()
    }
}

/// Format bytes as human-readable size
fn format_bytes(bytes: i64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    if bytes == 0 {
        return "0 B".to_string();
    }

    let bytes = bytes.abs() as f64;
    let i = (bytes.ln() / 1024_f64.ln()).floor() as usize;
    let i = i.min(UNITS.len() - 1);
    let size = bytes / 1024_f64.powi(i as i32);

    if i == 0 {
        format!("{:.0} {}", size, UNITS[i])
    } else {
        format!("{:.2} {}", size, UNITS[i])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::objects::{DatabaseObject, DatabaseObjectType};

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(0), "0 B");
        assert_eq!(format_bytes(1024), "1.00 KB");
        assert_eq!(format_bytes(1048576), "1.00 MB");
        assert_eq!(format_bytes(1073741824), "1.00 GB");
    }

    #[test]
    fn test_get_object_icon_and_color() {
        let (icon, color) = get_object_icon_and_color(&DatabaseObjectType::Table, Some(&DatabaseType::PostgreSQL));
        assert_eq!(icon, "🐘");
        assert_eq!(color, Color::Blue);

        let (icon, color) = get_object_icon_and_color(&DatabaseObjectType::Table, Some(&DatabaseType::MySQL));
        assert_eq!(icon, "🐬");
        assert_eq!(color, Color::Green);
    }

    #[test]
    fn test_get_qualified_name() {
        let obj = DatabaseObject {
            name: "users".to_string(),
            schema: Some("public".to_string()),
            object_type: DatabaseObjectType::Table,
            row_count: None,
            size_bytes: None,
            comment: None,
        };

        // PostgreSQL should show qualified name
        let qualified = get_qualified_name(&obj, Some(&DatabaseType::PostgreSQL));
        assert_eq!(qualified, "users"); // public schema is filtered out

        // MySQL should show schema.table
        let qualified = get_qualified_name(&obj, Some(&DatabaseType::MySQL));
        assert_eq!(qualified, "public.users");

        // SQLite should show just table name
        let qualified = get_qualified_name(&obj, Some(&DatabaseType::SQLite));
        assert_eq!(qualified, "users");
    }
}