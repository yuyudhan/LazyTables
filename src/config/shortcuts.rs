// FilePath: src/config/shortcuts.rs

use crate::core::error::Result;
use crossterm::event::{KeyCode, KeyModifiers};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, str::FromStr};

/// Keybindings configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeybindingsConfig {
    pub leader_key: String,
    /// Hotkeys for switching directly to panes
    pub pane_hotkeys: PaneHotkeys,
    /// Navigation hotkeys
    pub navigation: NavigationHotkeys,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaneHotkeys {
    /// Switch to Connections pane
    pub connections: String,
    /// Switch to Tables pane  
    pub tables: String,
    /// Switch to Table Details pane
    pub details: String,
    /// Switch to Tabular Output pane
    pub tabular_output: String,
    /// Switch to SQL Files pane
    pub sql_files: String,
    /// Switch to Query Window pane
    pub query_window: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavigationHotkeys {
    /// Move focus left
    pub focus_left: String,
    /// Move focus down
    pub focus_down: String,
    /// Move focus up
    pub focus_up: String,
    /// Move focus right
    pub focus_right: String,
    /// Cycle focus forward
    pub cycle_forward: String,
    /// Cycle focus backward
    pub cycle_backward: String,
}

/// Represents a key combination that can be bound to an action
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Hotkey {
    pub modifiers: KeyModifiers,
    pub key: KeyCode,
}

impl Hotkey {
    pub fn new(modifiers: KeyModifiers, key: KeyCode) -> Self {
        Self { modifiers, key }
    }

    /// Check if this hotkey matches a key event
    pub fn matches(&self, modifiers: KeyModifiers, key: KeyCode) -> bool {
        self.modifiers == modifiers && self.key == key
    }
}

impl FromStr for Hotkey {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('+').collect();
        if parts.is_empty() {
            return Err("Empty hotkey string".to_string());
        }

        let mut modifiers = KeyModifiers::NONE;
        let key_str = parts.last().unwrap();

        // Parse modifiers
        for part in &parts[..parts.len() - 1] {
            match part.to_lowercase().as_str() {
                "ctrl" | "control" => modifiers |= KeyModifiers::CONTROL,
                "alt" => modifiers |= KeyModifiers::ALT,
                "shift" => modifiers |= KeyModifiers::SHIFT,
                "super" | "cmd" | "meta" => modifiers |= KeyModifiers::SUPER,
                _ => return Err(format!("Unknown modifier: {}", part)),
            }
        }

        // Parse key
        let key = match key_str.to_lowercase().as_str() {
            "esc" | "escape" => KeyCode::Esc,
            "enter" | "return" => KeyCode::Enter,
            "space" => KeyCode::Char(' '),
            "tab" => KeyCode::Tab,
            "backspace" => KeyCode::Backspace,
            "delete" | "del" => KeyCode::Delete,
            "insert" | "ins" => KeyCode::Insert,
            "home" => KeyCode::Home,
            "end" => KeyCode::End,
            "pageup" | "pgup" => KeyCode::PageUp,
            "pagedown" | "pgdn" => KeyCode::PageDown,
            "up" => KeyCode::Up,
            "down" => KeyCode::Down,
            "left" => KeyCode::Left,
            "right" => KeyCode::Right,
            s if s.len() == 1 => KeyCode::Char(s.chars().next().unwrap()),
            s if s.starts_with('f') && s.len() <= 3 => {
                let num = s[1..].parse::<u8>().map_err(|_| format!("Invalid function key: {}", s))?;
                if num >= 1 && num <= 12 {
                    KeyCode::F(num)
                } else {
                    return Err(format!("Function key out of range: {}", s));
                }
            }
            _ => return Err(format!("Unknown key: {}", key_str)),
        };

        Ok(Hotkey { modifiers, key })
    }
}

impl ToString for Hotkey {
    fn to_string(&self) -> String {
        let mut parts = Vec::new();

        if self.modifiers.contains(KeyModifiers::CONTROL) {
            parts.push("Ctrl");
        }
        if self.modifiers.contains(KeyModifiers::ALT) {
            parts.push("Alt");
        }
        if self.modifiers.contains(KeyModifiers::SHIFT) {
            parts.push("Shift");
        }
        if self.modifiers.contains(KeyModifiers::SUPER) {
            parts.push("Super");
        }

        let key_str = match self.key {
            KeyCode::Esc => "Esc".to_string(),
            KeyCode::Enter => "Enter".to_string(),
            KeyCode::Char(' ') => "Space".to_string(),
            KeyCode::Char(c) => c.to_string(),
            KeyCode::Tab => "Tab".to_string(),
            KeyCode::Backspace => "Backspace".to_string(),
            KeyCode::Delete => "Delete".to_string(),
            KeyCode::Insert => "Insert".to_string(),
            KeyCode::Home => "Home".to_string(),
            KeyCode::End => "End".to_string(),
            KeyCode::PageUp => "PageUp".to_string(),
            KeyCode::PageDown => "PageDown".to_string(),
            KeyCode::Up => "Up".to_string(),
            KeyCode::Down => "Down".to_string(),
            KeyCode::Left => "Left".to_string(),
            KeyCode::Right => "Right".to_string(),
            KeyCode::F(n) => format!("F{}", n),
            _ => "Unknown".to_string(),
        };

        parts.push(&key_str);
        parts.join("+")
    }
}

/// Hotkey manager for handling configured hotkeys
pub struct HotkeyManager {
    pane_hotkeys: HashMap<Hotkey, crate::app::state::FocusedPane>,
    navigation_hotkeys: HashMap<Hotkey, NavigationAction>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavigationAction {
    FocusLeft,
    FocusDown,
    FocusUp,
    FocusRight,
    CycleForward,
    CycleBackward,
}

impl HotkeyManager {
    pub fn new(config: &KeybindingsConfig) -> Result<Self> {
        let mut pane_hotkeys = HashMap::new();
        let mut navigation_hotkeys = HashMap::new();

        // Parse pane hotkeys
        let pane_configs = [
            (&config.pane_hotkeys.connections, crate::app::state::FocusedPane::Connections),
            (&config.pane_hotkeys.tables, crate::app::state::FocusedPane::Tables),
            (&config.pane_hotkeys.details, crate::app::state::FocusedPane::Details),
            (&config.pane_hotkeys.tabular_output, crate::app::state::FocusedPane::TabularOutput),
            (&config.pane_hotkeys.sql_files, crate::app::state::FocusedPane::SqlFiles),
            (&config.pane_hotkeys.query_window, crate::app::state::FocusedPane::QueryWindow),
        ];

        for (hotkey_str, pane) in pane_configs {
            let hotkey = Hotkey::from_str(hotkey_str)
                .map_err(|e| crate::core::error::LazyTablesError::Config(format!("Invalid pane hotkey '{}': {}", hotkey_str, e)))?;
            pane_hotkeys.insert(hotkey, pane);
        }

        // Parse navigation hotkeys
        let nav_configs = [
            (&config.navigation.focus_left, NavigationAction::FocusLeft),
            (&config.navigation.focus_down, NavigationAction::FocusDown),
            (&config.navigation.focus_up, NavigationAction::FocusUp),
            (&config.navigation.focus_right, NavigationAction::FocusRight),
            (&config.navigation.cycle_forward, NavigationAction::CycleForward),
            (&config.navigation.cycle_backward, NavigationAction::CycleBackward),
        ];

        for (hotkey_str, action) in nav_configs {
            let hotkey = Hotkey::from_str(hotkey_str)
                .map_err(|e| crate::core::error::LazyTablesError::Config(format!("Invalid navigation hotkey '{}': {}", hotkey_str, e)))?;
            navigation_hotkeys.insert(hotkey, action);
        }

        Ok(Self {
            pane_hotkeys,
            navigation_hotkeys,
        })
    }

    /// Check if a key event matches a pane switching hotkey
    pub fn get_pane_for_key(&self, modifiers: KeyModifiers, key: KeyCode) -> Option<crate::app::state::FocusedPane> {
        self.pane_hotkeys
            .iter()
            .find(|(hotkey, _)| hotkey.matches(modifiers, key))
            .map(|(_, pane)| *pane)
    }

    /// Check if a key event matches a navigation hotkey
    pub fn get_navigation_action(&self, modifiers: KeyModifiers, key: KeyCode) -> Option<NavigationAction> {
        self.navigation_hotkeys
            .iter()
            .find(|(hotkey, _)| hotkey.matches(modifiers, key))
            .map(|(_, action)| *action)
    }

    /// Get all configured pane hotkeys for display
    pub fn get_pane_hotkeys(&self) -> &HashMap<Hotkey, crate::app::state::FocusedPane> {
        &self.pane_hotkeys
    }

    /// Get all configured navigation hotkeys for display
    pub fn get_navigation_hotkeys(&self) -> &HashMap<Hotkey, NavigationAction> {
        &self.navigation_hotkeys
    }
}

impl Default for KeybindingsConfig {
    fn default() -> Self {
        Self {
            leader_key: " ".to_string(),
            pane_hotkeys: PaneHotkeys {
                connections: "F1".to_string(),
                tables: "F2".to_string(),
                details: "F3".to_string(),
                tabular_output: "F4".to_string(),
                sql_files: "F5".to_string(),
                query_window: "F6".to_string(),
            },
            navigation: NavigationHotkeys {
                focus_left: "Ctrl+h".to_string(),
                focus_down: "Ctrl+j".to_string(),
                focus_up: "Ctrl+k".to_string(),
                focus_right: "Ctrl+l".to_string(),
                cycle_forward: "Tab".to_string(),
                cycle_backward: "Shift+Tab".to_string(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyModifiers};

    fn create_test_keybindings() -> KeybindingsConfig {
        KeybindingsConfig {
            leader_key: " ".to_string(),
            pane_hotkeys: PaneHotkeys {
                connections: "1".to_string(),
                tables: "2".to_string(),
                details: "3".to_string(),
                tabular_output: "4".to_string(),
                sql_files: "5".to_string(),
                query_window: "6".to_string(),
            },
            navigation: NavigationHotkeys {
                focus_left: "Alt+h".to_string(),
                focus_down: "Alt+j".to_string(),
                focus_up: "Alt+k".to_string(),
                focus_right: "Alt+l".to_string(),
                cycle_forward: "Tab".to_string(),
                cycle_backward: "Shift+Tab".to_string(),
            },
        }
    }

    #[test]
    fn test_keybindings_default() {
        let keybindings = KeybindingsConfig::default();
        
        assert_eq!(keybindings.leader_key, " ");
        assert_eq!(keybindings.pane_hotkeys.connections, "F1");
        assert_eq!(keybindings.pane_hotkeys.tables, "F2");
        assert_eq!(keybindings.navigation.focus_left, "Ctrl+h");
        assert_eq!(keybindings.navigation.cycle_forward, "Tab");
    }

    #[test]
    fn test_hotkey_from_str_single_char() {
        let hotkey = Hotkey::from_str("a").unwrap();
        assert_eq!(hotkey.modifiers, KeyModifiers::NONE);
        assert_eq!(hotkey.key, KeyCode::Char('a'));
    }

    #[test]
    fn test_hotkey_from_str_function_key() {
        let hotkey = Hotkey::from_str("F1").unwrap();
        assert_eq!(hotkey.modifiers, KeyModifiers::NONE);
        assert_eq!(hotkey.key, KeyCode::F(1));

        let hotkey = Hotkey::from_str("F12").unwrap();
        assert_eq!(hotkey.modifiers, KeyModifiers::NONE);
        assert_eq!(hotkey.key, KeyCode::F(12));
    }

    #[test]
    fn test_hotkey_from_str_special_keys() {
        let tests = vec![
            ("Enter", KeyCode::Enter),
            ("Tab", KeyCode::Tab),
            ("Esc", KeyCode::Esc),
            ("Space", KeyCode::Char(' ')),
            ("Backspace", KeyCode::Backspace),
            ("Delete", KeyCode::Delete),
            ("Up", KeyCode::Up),
            ("Down", KeyCode::Down),
            ("Left", KeyCode::Left),
            ("Right", KeyCode::Right),
        ];

        for (input, expected) in tests {
            let hotkey = Hotkey::from_str(input).unwrap();
            assert_eq!(hotkey.modifiers, KeyModifiers::NONE);
            assert_eq!(hotkey.key, expected);
        }
    }

    #[test]
    fn test_hotkey_from_str_with_modifiers() {
        let hotkey = Hotkey::from_str("Ctrl+a").unwrap();
        assert_eq!(hotkey.modifiers, KeyModifiers::CONTROL);
        assert_eq!(hotkey.key, KeyCode::Char('a'));

        let hotkey = Hotkey::from_str("Alt+F1").unwrap();
        assert_eq!(hotkey.modifiers, KeyModifiers::ALT);
        assert_eq!(hotkey.key, KeyCode::F(1));

        let hotkey = Hotkey::from_str("Shift+Tab").unwrap();
        assert_eq!(hotkey.modifiers, KeyModifiers::SHIFT);
        assert_eq!(hotkey.key, KeyCode::Tab);
    }

    #[test]
    fn test_hotkey_from_str_multiple_modifiers() {
        let hotkey = Hotkey::from_str("Ctrl+Shift+a").unwrap();
        assert_eq!(hotkey.modifiers, KeyModifiers::CONTROL | KeyModifiers::SHIFT);
        assert_eq!(hotkey.key, KeyCode::Char('a'));

        let hotkey = Hotkey::from_str("Ctrl+Alt+F5").unwrap();
        assert_eq!(hotkey.modifiers, KeyModifiers::CONTROL | KeyModifiers::ALT);
        assert_eq!(hotkey.key, KeyCode::F(5));
    }

    #[test]
    fn test_hotkey_from_str_case_insensitive() {
        let hotkey1 = Hotkey::from_str("ctrl+a").unwrap();
        let hotkey2 = Hotkey::from_str("CTRL+A").unwrap();
        let hotkey3 = Hotkey::from_str("Ctrl+a").unwrap();

        assert_eq!(hotkey1.modifiers, hotkey2.modifiers);
        assert_eq!(hotkey1.modifiers, hotkey3.modifiers);
        assert_eq!(hotkey1.key, KeyCode::Char('a'));
        assert_eq!(hotkey2.key, KeyCode::Char('a'));
    }

    #[test]
    fn test_hotkey_from_str_errors() {
        assert!(Hotkey::from_str("").is_err());
        assert!(Hotkey::from_str("InvalidModifier+a").is_err());
        assert!(Hotkey::from_str("Ctrl+InvalidKey").is_err());
        assert!(Hotkey::from_str("F13").is_err()); // Function key out of range
        assert!(Hotkey::from_str("F0").is_err());  // Function key out of range
    }

    #[test]
    fn test_hotkey_matches() {
        let hotkey = Hotkey::new(KeyModifiers::CONTROL, KeyCode::Char('a'));
        
        assert!(hotkey.matches(KeyModifiers::CONTROL, KeyCode::Char('a')));
        assert!(!hotkey.matches(KeyModifiers::NONE, KeyCode::Char('a')));
        assert!(!hotkey.matches(KeyModifiers::CONTROL, KeyCode::Char('b')));
        assert!(!hotkey.matches(KeyModifiers::ALT, KeyCode::Char('a')));
    }

    #[test]
    fn test_hotkey_to_string() {
        let hotkey = Hotkey::new(KeyModifiers::CONTROL, KeyCode::Char('a'));
        assert_eq!(hotkey.to_string(), "Ctrl+a");

        let hotkey = Hotkey::new(KeyModifiers::ALT | KeyModifiers::SHIFT, KeyCode::F(1));
        let string = hotkey.to_string();
        // Order may vary, but should contain both modifiers
        assert!(string.contains("Alt"));
        assert!(string.contains("Shift"));
        assert!(string.contains("F1"));

        let hotkey = Hotkey::new(KeyModifiers::NONE, KeyCode::Tab);
        assert_eq!(hotkey.to_string(), "Tab");
    }

    #[test]
    fn test_hotkey_roundtrip() {
        let test_strings = vec![
            "a", "F1", "Ctrl+a", "Alt+F1", "Shift+Tab", 
            "Ctrl+Shift+a", "Enter", "Space", "Up", "Down"
        ];

        for original in test_strings {
            let hotkey = Hotkey::from_str(original).unwrap();
            let back_to_string = hotkey.to_string();
            let hotkey2 = Hotkey::from_str(&back_to_string).unwrap();
            
            assert_eq!(hotkey.modifiers, hotkey2.modifiers);
            assert_eq!(hotkey.key, hotkey2.key);
        }
    }

    #[test]
    fn test_keybindings_serialization() {
        let keybindings = create_test_keybindings();
        let toml_str = toml::to_string_pretty(&keybindings).unwrap();
        
        // Verify TOML contains expected sections
        assert!(toml_str.contains("leader_key"));
        assert!(toml_str.contains("[pane_hotkeys]"));
        assert!(toml_str.contains("[navigation]"));
        assert!(toml_str.contains("connections = \"1\""));
        assert!(toml_str.contains("focus_left = \"Alt+h\""));
    }

    #[test]
    fn test_keybindings_deserialization() {
        let toml_str = r#"
leader_key = " "

[pane_hotkeys]
connections = "1"
tables = "2"
details = "3"
tabular_output = "4"
sql_files = "5"
query_window = "6"

[navigation]
focus_left = "Alt+h"
focus_down = "Alt+j"
focus_up = "Alt+k"
focus_right = "Alt+l"
cycle_forward = "Tab"
cycle_backward = "Shift+Tab"
"#.trim();

        let keybindings: KeybindingsConfig = toml::from_str(toml_str).unwrap();
        
        assert_eq!(keybindings.leader_key, " ");
        assert_eq!(keybindings.pane_hotkeys.connections, "1");
        assert_eq!(keybindings.pane_hotkeys.tables, "2");
        assert_eq!(keybindings.navigation.focus_left, "Alt+h");
        assert_eq!(keybindings.navigation.cycle_forward, "Tab");
    }

    #[test]
    fn test_hotkey_manager_creation() {
        let keybindings = create_test_keybindings();
        let manager = HotkeyManager::new(&keybindings).unwrap();
        
        // Verify pane hotkeys were parsed correctly
        let pane_hotkeys = manager.get_pane_hotkeys();
        assert_eq!(pane_hotkeys.len(), 6);
        
        // Verify navigation hotkeys were parsed correctly
        let nav_hotkeys = manager.get_navigation_hotkeys();
        assert_eq!(nav_hotkeys.len(), 6);
    }

    #[test]
    fn test_hotkey_manager_pane_lookup() {
        let keybindings = create_test_keybindings();
        let manager = HotkeyManager::new(&keybindings).unwrap();
        
        // Test pane switching
        assert_eq!(
            manager.get_pane_for_key(KeyModifiers::NONE, KeyCode::Char('1')),
            Some(crate::app::state::FocusedPane::Connections)
        );
        assert_eq!(
            manager.get_pane_for_key(KeyModifiers::NONE, KeyCode::Char('2')),
            Some(crate::app::state::FocusedPane::Tables)
        );
        
        // Test non-matching key
        assert_eq!(
            manager.get_pane_for_key(KeyModifiers::NONE, KeyCode::Char('9')),
            None
        );
    }

    #[test]
    fn test_hotkey_manager_navigation_lookup() {
        let keybindings = create_test_keybindings();
        let manager = HotkeyManager::new(&keybindings).unwrap();
        
        // Test navigation actions
        assert_eq!(
            manager.get_navigation_action(KeyModifiers::ALT, KeyCode::Char('h')),
            Some(NavigationAction::FocusLeft)
        );
        assert_eq!(
            manager.get_navigation_action(KeyModifiers::ALT, KeyCode::Char('j')),
            Some(NavigationAction::FocusDown)
        );
        assert_eq!(
            manager.get_navigation_action(KeyModifiers::NONE, KeyCode::Tab),
            Some(NavigationAction::CycleForward)
        );
        
        // Test non-matching key
        assert_eq!(
            manager.get_navigation_action(KeyModifiers::NONE, KeyCode::Char('x')),
            None
        );
    }

    #[test]
    fn test_hotkey_manager_invalid_config() {
        let invalid_keybindings = KeybindingsConfig {
            leader_key: " ".to_string(),
            pane_hotkeys: PaneHotkeys {
                connections: "InvalidKey".to_string(),
                tables: "F2".to_string(),
                details: "F3".to_string(),
                tabular_output: "F4".to_string(),
                sql_files: "F5".to_string(),
                query_window: "F6".to_string(),
            },
            navigation: NavigationHotkeys {
                focus_left: "Ctrl+h".to_string(),
                focus_down: "Ctrl+j".to_string(),
                focus_up: "Ctrl+k".to_string(),
                focus_right: "Ctrl+l".to_string(),
                cycle_forward: "Tab".to_string(),
                cycle_backward: "Shift+Tab".to_string(),
            },
        };

        let result = HotkeyManager::new(&invalid_keybindings);
        assert!(result.is_err());
    }

    #[test]
    fn test_navigation_action_debug() {
        let action = NavigationAction::FocusLeft;
        let debug_str = format!("{:?}", action);
        assert_eq!(debug_str, "FocusLeft");
    }

    #[test]
    fn test_pane_hotkeys_all_fields() {
        let pane_hotkeys = PaneHotkeys {
            connections: "F1".to_string(),
            tables: "F2".to_string(),
            details: "F3".to_string(),
            tabular_output: "F4".to_string(),
            sql_files: "F5".to_string(),
            query_window: "F6".to_string(),
        };

        // Verify all fields are accessible
        assert_eq!(pane_hotkeys.connections, "F1");
        assert_eq!(pane_hotkeys.tables, "F2");
        assert_eq!(pane_hotkeys.details, "F3");
        assert_eq!(pane_hotkeys.tabular_output, "F4");
        assert_eq!(pane_hotkeys.sql_files, "F5");
        assert_eq!(pane_hotkeys.query_window, "F6");
    }

    #[test]
    fn test_navigation_hotkeys_all_fields() {
        let nav_hotkeys = NavigationHotkeys {
            focus_left: "Ctrl+h".to_string(),
            focus_down: "Ctrl+j".to_string(),
            focus_up: "Ctrl+k".to_string(),
            focus_right: "Ctrl+l".to_string(),
            cycle_forward: "Tab".to_string(),
            cycle_backward: "Shift+Tab".to_string(),
        };

        // Verify all fields are accessible
        assert_eq!(nav_hotkeys.focus_left, "Ctrl+h");
        assert_eq!(nav_hotkeys.focus_down, "Ctrl+j");
        assert_eq!(nav_hotkeys.focus_up, "Ctrl+k");
        assert_eq!(nav_hotkeys.focus_right, "Ctrl+l");
        assert_eq!(nav_hotkeys.cycle_forward, "Tab");
        assert_eq!(nav_hotkeys.cycle_backward, "Shift+Tab");
    }

    #[test]
    fn test_hotkey_equality() {
        let hotkey1 = Hotkey::new(KeyModifiers::CONTROL, KeyCode::Char('a'));
        let hotkey2 = Hotkey::new(KeyModifiers::CONTROL, KeyCode::Char('a'));
        let hotkey3 = Hotkey::new(KeyModifiers::ALT, KeyCode::Char('a'));

        assert_eq!(hotkey1, hotkey2);
        assert_ne!(hotkey1, hotkey3);
    }

    #[test]
    fn test_hotkey_hash() {
        use std::collections::HashSet;
        
        let hotkey1 = Hotkey::new(KeyModifiers::CONTROL, KeyCode::Char('a'));
        let hotkey2 = Hotkey::new(KeyModifiers::CONTROL, KeyCode::Char('a'));
        let hotkey3 = Hotkey::new(KeyModifiers::ALT, KeyCode::Char('a'));

        let mut set = HashSet::new();
        set.insert(hotkey1);
        set.insert(hotkey2); // Should not add duplicate
        set.insert(hotkey3);

        assert_eq!(set.len(), 2); // Only hotkey1 and hotkey3 should be in set
    }
}