//! Color constants for consistent theming

use ratatui::style::Color;

// Backgrounds for cursor and selection
pub const CURSOR_BG: Color = Color::Rgb(40, 40, 60);
pub const SOURCE_BG: Color = Color::Rgb(50, 50, 30);
pub const SELECTED_BG: Color = Color::Rgb(40, 50, 40);

// Status bar
pub const STATUS_BAR_BG: Color = Color::Rgb(30, 30, 50);

// Popups
pub const POPUP_BG: Color = Color::Rgb(20, 20, 30);
pub const POPUP_BG_DELETE: Color = Color::Rgb(30, 20, 20);
pub const PREFIX_POPUP_BG: Color = Color::Rgb(25, 25, 35);
pub const TOAST_BG: Color = Color::Rgb(30, 30, 40);

// Diff backgrounds
pub const DIFF_ADDED_BG: Color = Color::Rgb(0, 40, 0);
pub const DIFF_REMOVED_BG: Color = Color::Rgb(40, 0, 0);
