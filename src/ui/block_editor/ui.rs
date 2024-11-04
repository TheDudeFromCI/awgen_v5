//! This module handles the construction of the Block Editor UI screen within
//! the editor mode.

use bevy::prelude::*;
use bevy_egui::EguiContexts;
use bevy_egui::egui::{self, Color32, Frame, Margin, Rounding, Stroke};

use super::helper::{BlockEditHelper, Popup};
use super::preview::BlockPreviewWidget;
use crate::ui::EditorWindowState;

/// Builds the Block Editor UI screen.
pub fn render(
    mut block_edit_helper: BlockEditHelper,
    mut preview_widget: ResMut<BlockPreviewWidget>,
    mut contexts: EguiContexts,
) {
    block_edit_helper.initialize();

    let block_preview_texture_id = contexts.image_id(&preview_widget.get_handle()).unwrap();

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

    egui::SidePanel::right("tileset_panel")
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

                    // TODO: Render tileset here.
                });
        });

    let main_panel_rect = egui::CentralPanel::default()
        .show(ctx, |ui| {
            if block_edit_helper.is_popup_open() {
                ui.disable();
            }

            block_edit_helper.edit_name(ui);

            let preview_size = preview_widget.get_size() as f32;
            let block_preview_response = ui.image(egui::load::SizedTexture::new(
                block_preview_texture_id,
                egui::vec2(preview_size, preview_size),
            ));

            let cam_rot = block_preview_response
                .interact(egui::Sense::drag())
                .drag_delta();
            preview_widget.rotate(-cam_rot.x, -cam_rot.y);

            preview_widget.set_mouse_pos(
                block_preview_response
                    .interact(egui::Sense::hover())
                    .hover_pos()
                    .map(|pos| pos - block_preview_response.rect.min)
                    .map(|pos| Vec2::new(pos.x, pos.y))
                    .unwrap_or_default(),
            );

            if block_preview_response
                .interact(egui::Sense::click())
                .clicked()
            {
                let face = preview_widget.get_hovered_face();
                preview_widget.set_selected_face(face);
                debug!("Selected face: {:?}", face);
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

    preview_widget.set_active_block(block_edit_helper.selected_block());
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
    block_edit_helper: BlockEditHelper,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut editor_window_state: ResMut<NextState<EditorWindowState>>,
) {
    if keyboard_input.just_pressed(KeyCode::F1) || keyboard_input.just_pressed(KeyCode::Escape) {
        if block_edit_helper.is_popup_open() {
            // Do not close the window if a popup is open.
            return;
        }

        editor_window_state.set(EditorWindowState::MapEditor);
        info!("Closed Block Editor UI window.");
    }
}
