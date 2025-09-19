// Database object types and representations

use serde::{Deserialize, Serialize};

/// Type of database object
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DatabaseObjectType {
    Table,
    View,
    MaterializedView,
    ForeignTable,
    SystemTable,
}

impl DatabaseObjectType {
    /// Get icon for this object type
    pub fn icon(&self) -> &str {
        match self {
            Self::Table => "📋",
            Self::View => "👁️",
            Self::MaterializedView => "🔄",
            Self::ForeignTable => "🔗",
            Self::SystemTable => "⚙️",
        }
    }

    /// Get display name for this type
    pub fn display_name(&self) -> &str {
        match self {
            Self::Table => "Table",
            Self::View => "View",
            Self::MaterializedView => "Materialized View",
            Self::ForeignTable => "Foreign Table",
            Self::SystemTable => "System Table",
        }
    }
}

/// Represents a database object (table, view, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseObject {
    pub name: String,
    pub schema: Option<String>,
    pub object_type: DatabaseObjectType,
    pub row_count: Option<i64>,
    pub size_bytes: Option<i64>,
    pub comment: Option<String>,
}

impl DatabaseObject {
    /// Get full qualified name (schema.name or just name)
    pub fn qualified_name(&self) -> String {
        if let Some(schema) = &self.schema {
            if schema != "public" {
                return format!("{}.{}", schema, self.name);
            }
        }
        self.name.clone()
    }

    /// Check if this is a system object
    pub fn is_system(&self) -> bool {
        matches!(self.object_type, DatabaseObjectType::SystemTable)
            || self.name.starts_with("pg_")
            || self.name.starts_with("information_schema")
            || self.schema.as_deref() == Some("pg_catalog")
            || self.schema.as_deref() == Some("information_schema")
    }
}

/// Result of listing database objects
#[derive(Debug, Clone, Default)]
pub struct DatabaseObjectList {
    pub tables: Vec<DatabaseObject>,
    pub views: Vec<DatabaseObject>,
    pub materialized_views: Vec<DatabaseObject>,
    pub foreign_tables: Vec<DatabaseObject>,
    pub total_count: usize,
    pub error: Option<String>,
}

impl DatabaseObjectList {
    /// Get all objects as a flat list
    pub fn all_objects(&self) -> Vec<&DatabaseObject> {
        self.tables
            .iter()
            .chain(self.views.iter())
            .chain(self.materialized_views.iter())
            .chain(self.foreign_tables.iter())
            .collect()
    }

    /// Filter objects by name pattern
    pub fn filter(&self, pattern: &str) -> Vec<&DatabaseObject> {
        let pattern_lower = pattern.to_lowercase();
        self.all_objects()
            .into_iter()
            .filter(|obj| obj.name.to_lowercase().contains(&pattern_lower))
            .collect()
    }

    /// Check if the list is empty
    pub fn is_empty(&self) -> bool {
        self.tables.is_empty()
            && self.views.is_empty()
            && self.materialized_views.is_empty()
            && self.foreign_tables.is_empty()
    }
}


