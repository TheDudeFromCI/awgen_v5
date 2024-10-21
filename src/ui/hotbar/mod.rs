//! This module adds functionality for creating the editor hotbar HUD element.

use bevy::asset::embedded_asset;
use bevy::prelude::*;
use resource::Hotbar;

use super::menu::MainMenuState;

pub mod resource;
pub mod systems;

/// The asset path to the editor hotbar background image.
const HOTBAR_BG_IMG: &str = "embedded://awgen/ui/hotbar/bg.png";

/// The asset path to the editor hotbar selection image.
const HOTBAR_SEL_IMG: &str = "embedded://awgen/ui/hotbar/selection.png";

/// The pixel size of a single hotbar element.
const HOTBAR_SIZE: f32 = 48.0;

/// The number of pixels between each hotbar element.
const HOTBAR_GAP: f32 = 2.0;

/// This plugin adds the editor hotbar systems and components to the app.
pub struct UiHotbarPlugin;
impl Plugin for UiHotbarPlugin {
    fn build(&self, app_: &mut App) {
        app_.init_resource::<Hotbar>()
            .add_systems(
                OnEnter(MainMenuState::Editor),
                systems::setup_hotbar
                    .before_ignore_deferred(crate::map::editor::startup::prepare_map_editor),
            )
            .add_systems(OnExit(MainMenuState::Editor), systems::cleanup_hotbar)
            .add_systems(
                Update,
                (
                    systems::select_slot_with_numkeys.run_if(in_state(MainMenuState::Editor)),
                    systems::update_selected_index
                        .run_if(in_state(MainMenuState::Editor))
                        .run_if(resource_changed::<Hotbar>)
                        .after_ignore_deferred(systems::select_slot_with_numkeys),
                    systems::update_slot_visuals
                        .run_if(in_state(MainMenuState::Editor))
                        .run_if(resource_changed::<Hotbar>)
                        .after_ignore_deferred(systems::update_selected_index),
                ),
            );

        embedded_asset!(app_, "bg.png");
        embedded_asset!(app_, "selection.png");
    }
}

/// This is a marker component used to indicate that the entity is the root of
/// the hotbar.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Component)]
pub struct HotbarRoot;

/// This is a marker component used to indicate that the entity is a hotbar
/// slot.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Component)]
pub struct HotbarSlot;

/// This is a marker component used to indicate that the entity is the hotbar
/// selection element.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Component)]
pub struct HotbarSelector;
