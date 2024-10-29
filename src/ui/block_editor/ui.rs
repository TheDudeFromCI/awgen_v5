//! This module handles the construction of the Block Editor UI screen within
//! the editor mode.

use bevy::prelude::*;
use bevy_egui::EguiContexts;
use bevy_egui::egui::{self, Color32, Frame};

use crate::ui::EditorWindowState;

/// Builds the Block Editor UI screen.
pub fn build(mut contexts: EguiContexts) {
    let ctx = contexts.ctx_mut();

    egui::SidePanel::left("block_list_panel")
        .default_width(200.0)
        .min_width(100.0)
        .resizable(true)
        .frame(Frame {
            fill: Color32::from_gray(20),
            ..default()
        })
        .show(ctx, |ui| {
            egui::ScrollArea::vertical()
                .id_salt("block_list_scroll")
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());

                    for i in 0 .. 100 {
                        ui.label(format!("Block {}", i));
                    }
                });
        });

    egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading("Grass Block");
        ui.label("This is the Block Editor UI screen.");
    });
}

/// This system transitions to the Block Editor UI screen.
pub fn open(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut editor_window_state: ResMut<NextState<EditorWindowState>>,
) {
    if keyboard_input.just_pressed(KeyCode::F1) {
        editor_window_state.set(EditorWindowState::BlockEditor);
        info!("Opened Block Editor UI window.");
    }
}

/// This system closes the Block Editor UI screen and returns to the Map Editor.
pub fn close(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut editor_window_state: ResMut<NextState<EditorWindowState>>,
) {
    if keyboard_input.just_pressed(KeyCode::F1) || keyboard_input.just_pressed(KeyCode::Escape) {
        editor_window_state.set(EditorWindowState::MapEditor);
        info!("Closed Block Editor UI window.");
    }
}
