// FilePath: crates/lazytables-core/src/ui.rs

// Shared UI types: confirmation modals, pane focus management, help mode,
// and selectable table items used by both state and rendering code.

use crate::objects::DatabaseObjectType;
use serde::{Deserialize, Serialize};

/// Confirmation modal for destructive actions
#[derive(Debug, Clone)]
pub struct ConfirmationModal {
    pub title: String,
    pub message: String,
    pub action: ConfirmationAction,
}

/// Actions that can be confirmed
#[derive(Debug, Clone)]
pub enum ConfirmationAction {
    DeleteConnection(usize),
    DeleteTable(String),
    DeleteSqlFile(usize),
    ExitApplication,
    QuitQueryEditor,
}

/// Which pane currently has focus
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FocusedPane {
    /// Connections list pane
    Connections,
    /// Tables/Views list pane
    Tables,
    /// Table details pane
    Details,
    /// Tabular output area
    TabularOutput,
    /// SQL file browser
    SqlFiles,
    /// Query window (SQL editor)
    QueryWindow,
}

impl FocusedPane {
    /// Get the next pane in clockwise order
    pub fn next(&self) -> Self {
        match self {
            Self::Connections => Self::Tables,
            Self::Tables => Self::Details,
            Self::Details => Self::TabularOutput,
            Self::TabularOutput => Self::SqlFiles,
            Self::SqlFiles => Self::QueryWindow,
            Self::QueryWindow => Self::Connections,
        }
    }

    /// Get the previous pane in counter-clockwise order
    pub fn previous(&self) -> Self {
        match self {
            Self::Connections => Self::QueryWindow,
            Self::Tables => Self::Connections,
            Self::Details => Self::Tables,
            Self::TabularOutput => Self::Details,
            Self::SqlFiles => Self::TabularOutput,
            Self::QueryWindow => Self::SqlFiles,
        }
    }

    /// Get the pane number (1-6) for display
    pub fn to_number(&self) -> u8 {
        match self {
            Self::Connections => 1,
            Self::Tables => 2,
            Self::Details => 3,
            Self::TabularOutput => 4,
            Self::QueryWindow => 5,
            Self::SqlFiles => 6,
        }
    }

    /// Create a pane from a number (1-6)
    pub fn from_number(num: u8) -> Option<Self> {
        match num {
            1 => Some(Self::Connections),
            2 => Some(Self::Tables),
            3 => Some(Self::Details),
            4 => Some(Self::TabularOutput),
            5 => Some(Self::QueryWindow),
            6 => Some(Self::SqlFiles),
            _ => None,
        }
    }

    /// Get the display name of the pane
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Connections => "Connections",
            Self::Tables => "Tables",
            Self::Details => "Table Details",
            Self::TabularOutput => "Query Results",
            Self::QueryWindow => "SQL Query Editor",
            Self::SqlFiles => "SQL Files",
        }
    }
}

/// Help display mode for context-aware help
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HelpMode {
    /// No help displayed
    None,
    /// Connections pane help
    Connections,
    /// Tables pane help
    Tables,
    /// Details pane help
    Details,
    /// Tabular output help
    TabularOutput,
    /// SQL Files help
    SqlFiles,
    /// Query window help
    QueryWindow,
}

/// Which pane is focused in the help modal
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HelpPaneFocus {
    /// Left pane (pane-specific help)
    Left,
    /// Right pane (global commands)
    Right,
}

/// Represents a selectable item in the tables pane
#[derive(Debug, Clone)]
pub struct SelectableTableItem {
    /// The display name of the item
    pub display_name: String,
    /// The actual table/view name for database operations
    pub object_name: String,
    /// The schema this object belongs to (if any)
    pub schema: Option<String>,
    /// Type of database object
    pub object_type: DatabaseObjectType,
    /// Whether this item is selectable (false for headers)
    pub is_selectable: bool,
    /// The index of this item in the display list
    pub display_index: usize,
    /// If this is a schema header, contains the schema name for expansion toggle
    pub schema_header_name: Option<String>,
}

impl SelectableTableItem {
    /// Create a new selectable table item
    pub fn new_selectable(
        display_name: String,
        object_name: String,
        schema: Option<String>,
        object_type: DatabaseObjectType,
        display_index: usize,
    ) -> Self {
        Self {
            display_name,
            object_name,
            schema,
            object_type,
            is_selectable: true,
            display_index,
            schema_header_name: None,
        }
    }

    /// Create a non-selectable header item
    pub fn new_header(display_name: String, display_index: usize) -> Self {
        Self {
            display_name,
            object_name: String::new(),
            schema: None,
            object_type: DatabaseObjectType::Table,
            is_selectable: false,
            display_index,
            schema_header_name: None,
        }
    }

    /// Create a schema header item (non-selectable, but toggleable)
    pub fn new_schema_header(
        display_name: String,
        schema_name: String,
        display_index: usize,
    ) -> Self {
        Self {
            display_name,
            object_name: String::new(),
            schema: Some(schema_name.clone()),
            object_type: DatabaseObjectType::Table,
            is_selectable: false,
            display_index,
            schema_header_name: Some(schema_name),
        }
    }

    /// Check if this is a schema header
    pub fn is_schema_header(&self) -> bool {
        self.schema_header_name.is_some()
    }

    /// Get the qualified name for database operations
    pub fn qualified_name(&self) -> String {
        if let Some(ref schema) = self.schema {
            format!("{}.{}", schema, self.object_name)
        } else {
            self.object_name.clone()
        }
    }
}
