//! This module handles the construction of the Block Editor UI screen within
//! the editor mode.

use bevy::prelude::*;
use bevy_egui::EguiContexts;
use bevy_egui::egui::{self, Color32, Frame, Margin, Rounding, Stroke};

use super::preview::{BlockPreviewElement, BlockPreviewWidget};
use super::temp::{BlockEditHelper, Popup};
use crate::ui::EditorWindowState;

/// Builds the Block Editor UI screen.
pub fn render(
    mut block_edit_helper: BlockEditHelper,
    mut block_preview_widget: ResMut<BlockPreviewWidget>,
    mut contexts: EguiContexts,
    mut block_preview_camera: Query<&mut Transform, (With<Camera>, With<BlockPreviewElement>)>,
) {
    block_edit_helper.initialize();

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
            if block_edit_helper.is_popup_open() {
                ui.disable();
            }

            egui::ScrollArea::vertical()
                .id_salt("block_list_scroll")
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    block_edit_helper.edit_block_list(ui);
                });
        });

    let main_panel_rect = egui::CentralPanel::default()
        .show(ctx, |ui| {
            if block_edit_helper.is_popup_open() {
                ui.disable();
            }

            block_edit_helper.edit_name(ui);

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
        })
        .response
        .rect;

    let popup_size = (250.0, 100.0);
    let popup_pos = main_panel_rect.center() - egui::vec2(popup_size.0 / 2.0, popup_size.1 / 2.0);

    match block_edit_helper.get_popup() {
        Popup::None => {}

        Popup::UnsavedChanges { new_block } => {
            egui::Window::new("Unsaved Changes")
                .resizable(false)
                .collapsible(false)
                .title_bar(false)
                .fixed_size(popup_size)
                .default_pos(popup_pos)
                .frame(Frame {
                    inner_margin: Margin::same(10.0),
                    fill: Color32::from_gray(35),
                    rounding: Rounding::same(6.0),
                    stroke: Stroke {
                        width: 3.0,
                        color: Color32::from_gray(100),
                    },
                    ..default()
                })
                .show(ctx, |ui| {
                    ui.heading("Warning");
                    ui.label("You have unsaved changes. Do you want to save them?");

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::BOTTOM), |ui| {
                        ui.set_row_height(ui.available_height());

                        if ui.button("Discard").clicked() {
                            block_edit_helper.select_block(new_block);
                            block_edit_helper.close_popup();
                        }

                        if ui.button("Save").clicked() {
                            block_edit_helper.save_block();
                            block_edit_helper.select_block(new_block);
                            block_edit_helper.close_popup();
                        }
                    });
                });
        }
    }

    block_preview_widget.active_block = block_edit_helper.selected_block();
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
