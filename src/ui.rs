//! The screen: components that each own their state and draw themselves.
//!
//! Panels are told apart by background tint, never borders, and every color comes from
//! [`mod@theme`]. See `docs/conventions/ui.md`.

pub mod card;
pub mod composer;
pub mod dates;
pub mod envelope_list;
pub mod help;
pub mod layout;
pub mod movement;
pub mod overlay;
pub mod reader;
pub mod sidebar;
pub mod status;
pub mod theme;
pub mod topbar;

pub use self::theme::theme;
