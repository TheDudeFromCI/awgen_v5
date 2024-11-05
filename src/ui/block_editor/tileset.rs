//! This module implements the tileset list widget within the Block Editor UI
//! screen.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy_egui::egui::epaint;
use bevy_egui::{EguiContexts, egui};

use crate::blocks::Block;
use crate::blocks::tileset::{PROTOTYPE_TILESET_UUID, TilePos, Tileset};

/// The local data for the tileset list widget.
#[derive(Debug)]
pub struct TileListWidgetData {
    /// The currently selected tileset.
    pub tileset: Entity,
}

impl Default for TileListWidgetData {
    fn default() -> Self {
        Self {
            tileset: Entity::PLACEHOLDER,
        }
    }
}

/// The system parameter for the tileset list widget.
#[derive(SystemParam)]
pub struct TileListWidget<'w, 's> {
    /// The local data for the tileset list widget.
    data: Local<'s, TileListWidgetData>,

    /// The tileset entities query.
    tilesets: Query<
        'w,
        's,
        (
            Entity,
            &'static Name,
            &'static Tileset,
            &'static Handle<Image>,
        ),
        Without<Block>,
    >,
}

impl<'w, 's> TileListWidget<'w, 's> {
    /// This function should be called once per frame before using the widget to
    /// ensure that the widget is properly initialized. If the widget is already
    /// initialized, this function does nothing.
    pub fn initialize(&mut self, contexts: &mut EguiContexts) {
        if self.data.tileset != Entity::PLACEHOLDER {
            return;
        }

        let tileset = self
            .tilesets
            .iter()
            .find(|(_, _, tileset, _)| tileset.uuid == PROTOTYPE_TILESET_UUID)
            .map(|(entity, _, _, _)| entity)
            .expect("Prototype tileset should always be present.");

        self.set_tileset(tileset, contexts);
    }

    /// Sets the currently selected tileset.
    pub fn set_tileset(&mut self, tileset: Entity, contexts: &mut EguiContexts) {
        if tileset == self.data.tileset {
            return;
        }

        if let Ok((_, _, _, old_image)) = self.tilesets.get(tileset) {
            contexts.remove_image(old_image);
        };

        self.data.tileset = tileset;

        let (_, _, _, new_image) = self.tilesets.get(tileset).expect("Tileset should exist.");
        contexts.add_image(new_image.clone_weak());
    }

    /// Gets the image handle of the currently selected tileset.
    pub fn get_tileset_handle(&self) -> &Handle<Image> {
        let (_, _, _, image) = self.tilesets.get(self.data.tileset).unwrap();
        image
    }

    /// Gets the name of the currently selected tileset.
    pub fn get_tileset_name(&self) -> &Name {
        let (_, name, _, _) = self.tilesets.get(self.data.tileset).unwrap();
        name
    }
}

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

        // if response.clicked() {
        //     // TODO: Handle tile selection.
        // }

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
