//! This module implements dynamic tools for the editor.

use bevy::prelude::*;

/// The plugin for the tools components and functionality.
pub struct ToolsPlugin;
impl Plugin for ToolsPlugin {
    fn build(&self, app_: &mut App) {}
}

/// This is a marker component that indicates that an entity is a tool
/// definition.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Component)]
pub struct Tool;

/// A bundle used when initializing a tool entity.
#[derive(Debug, Default, Bundle)]
pub struct ToolBundle {
    /// The tool marker component.
    pub tool: Tool,

    /// The name of the tool.
    pub name: Name,

    /// The icon of the tool.
    pub icon: UiImage,
}
