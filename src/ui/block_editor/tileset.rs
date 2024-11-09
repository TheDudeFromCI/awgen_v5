//! This module implements the tileset list widget within the Block Editor UI
//! screen.

use bevy::prelude::*;
use bevy_egui::egui;
use bevy_egui::egui::epaint;

use crate::blocks::tileset::TilePos;

/// A simple EGUI widget that renders a single tile in the tileset.
pub struct TileWidget {
    /// The tileset image texture.
    pub texture: epaint::TextureId,

    /// The position of the tile in the tileset.
    pub tile_pos: TilePos,

    /// The size of the tile in pixels. The tile will be rendered as a square.
    pub size: f32,
}

impl egui::Widget for TileWidget {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let desired_size = egui::vec2(self.size, self.size);
        let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::click());

        if !ui.is_rect_visible(rect) {
            return response;
        }

        let center = (rect.max + rect.min.to_vec2()) / 2.0;

        ui.painter().image(
            self.texture,
            egui::Rect {
                min: (center - desired_size / 2.0).max(rect.min),
                max: (center + desired_size / 2.0).min(rect.max),
            },
            egui::Rect {
                min: pos(self.tile_pos.transform_uv(Vec2::ZERO)),
                max: pos(self.tile_pos.transform_uv(Vec2::ONE)),
            },
            egui::Color32::WHITE,
        );

        response
    }
}

/// A simple utility function for converting a Bevy `Vec2` to an EGUI `Pos2`.
#[inline(always)]
fn pos(value: Vec2) -> egui::Pos2 {
    egui::pos2(value.x, value.y)
}
