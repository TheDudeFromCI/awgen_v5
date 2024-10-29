//! This module handles the construction of the Block Editor UI screen within
//! the editor mode.

use bevy::prelude::*;
use bevy_egui::EguiContexts;
use bevy_egui::egui::{self, Color32, Frame, RichText};

use super::preview::{BlockPreviewElement, BlockPreviewWidget};
use crate::blocks::Block;
use crate::ui::EditorWindowState;

/// Builds the Block Editor UI screen.
pub fn build(
    mut block_preview_widget: ResMut<BlockPreviewWidget>,
    mut contexts: EguiContexts,
    mut block_preview_camera: Query<&mut Transform, (With<Camera>, With<BlockPreviewElement>)>,
    blocks: Query<(Entity, &Name), With<Block>>,
) {
    let block_preview_texture_id = contexts.image_id(&block_preview_widget.handle).unwrap();
    let mut block_preview_camera_transform = block_preview_camera.single_mut();

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

                    let block_list = blocks.iter().sort_by::<&Name>(|a, b| a.cmp(b));
                    for (block_id, name) in block_list {
                        ui.selectable_value(
                            &mut block_preview_widget.active_block,
                            block_id,
                            RichText::new(name).monospace().size(20.0),
                        );
                    }
                });
        });

    egui::CentralPanel::default().show(ctx, |ui| {
        let (_, name) = blocks.get(block_preview_widget.active_block).unwrap();

        ui.heading(&**name);
        ui.label("This is the Block Editor UI screen.");

        let block_preview_size = block_preview_widget.size as f32;
        let block_preview_response = ui
            .image(egui::load::SizedTexture::new(
                block_preview_texture_id,
                egui::vec2(block_preview_size, block_preview_size),
            ))
            .interact(egui::Sense::drag());

        let cam_rot = block_preview_response.drag_delta();
        if cam_rot != egui::Vec2::ZERO {
            block_preview_widget.rotate(-cam_rot.x, -cam_rot.y);
            block_preview_camera_transform.rotation = block_preview_widget.get_rotation();
        }
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
