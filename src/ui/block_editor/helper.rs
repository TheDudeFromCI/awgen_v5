//! This module handles storage and editing for the temporary block data that is
//! actively being edited.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy_egui::EguiContexts;
use bevy_egui::egui::{self, FontFamily, FontId, RichText};

use super::tileset::TileWidget;
use crate::blocks::shape::{BlockFace, BlockShape};
use crate::blocks::tileset::{TILESET_LENGTH, TilePos, Tileset};
use crate::blocks::{AIR_BLOCK_UUID, Block};
use crate::math::FaceDirection;

/// The data structure that holds the temporary block data that is being edited.
pub struct BlockEditData {
    /// The block entity that is being edited.
    pub block_id: Entity,

    /// Whether the block is dirty and needs to be saved.
    pub dirty: bool,

    /// The current popup that is being displayed.
    pub popup: Popup,

    /// The name of the block.
    pub name: String,
}

impl Default for BlockEditData {
    fn default() -> Self {
        Self {
            block_id: Entity::PLACEHOLDER,
            dirty: false,
            popup: Popup::None,
            name: String::new(),
        }
    }
}

/// A system parameter that helps with storing and editing block data.
#[derive(SystemParam)]
pub struct BlockEditHelper<'w, 's> {
    /// The block data cache that is being edited.
    data: Local<'s, BlockEditData>,

    /// The query that fetches all blocks.
    blocks: Query<
        'w,
        's,
        (
            Entity,
            &'static mut Name,
            &'static Block,
            &'static mut BlockShape,
        ),
        Without<Tileset>,
    >,

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

impl<'w, 's> BlockEditHelper<'w, 's> {
    /// This function is used to ensure that the block data is initialized. If
    /// the data is already initialized, this function does nothing.
    pub fn initialize(&mut self, contexts: &mut EguiContexts) {
        if self.data.block_id != Entity::PLACEHOLDER {
            return;
        }

        self.tilesets.iter().for_each(|(_, _, _, handle)| {
            contexts.add_image(handle.clone_weak());
        });

        let air = self
            .blocks
            .iter()
            .find(|(_, _, block, _)| block.uuid == AIR_BLOCK_UUID)
            .map(|(entity, _, _, _)| entity)
            .unwrap();

        self.select_block(air);
    }

    /// Adds a selectable list of all blocks to the UI.
    pub fn edit_block_list(&mut self, ui: &mut egui::Ui) {
        let block_list = self.blocks.iter().sort_by::<&Name>(|a, b| a.cmp(b));

        let mut sel_block = self.data.block_id;
        for (block_id, name, _, _) in block_list {
            ui.selectable_value(
                &mut sel_block,
                block_id,
                RichText::new(name).monospace().size(20.0),
            );
        }

        if sel_block != self.data.block_id {
            if self.data.dirty {
                self.data.popup = Popup::UnsavedChanges {
                    new_block: sel_block,
                };
            } else {
                self.select_block(sel_block);
            }
        }
    }

    /// Returns the current popup that is being displayed.
    pub fn get_popup(&self) -> Popup {
        self.data.popup
    }

    /// Updates the data to reflect a newly selected block. All other data is
    /// cleared and replaced, and this variable is marked as not dirty.
    pub fn select_block(&mut self, block: Entity) {
        self.data.block_id = block;
        self.data.dirty = false;

        let (_, name, _, _) = self.blocks.get(block).unwrap();
        self.data.name = name.as_str().to_string();
    }

    /// Adds a name edit field to the UI.
    pub fn edit_name(&mut self, ui: &mut egui::Ui) {
        ui.add(
            egui::TextEdit::singleline(&mut self.data.name)
                .char_limit(50)
                .font(FontId {
                    size: 32.0,
                    family: FontFamily::Monospace,
                }),
        );

        let original_name = self.blocks.get(self.data.block_id).unwrap().1;
        if self.data.name != original_name.as_str() {
            self.data.dirty = true;
        }
    }

    /// Saves the current block data.
    pub fn save_block(&mut self) {
        let (_, mut name, _, _) = self.blocks.get_mut(self.data.block_id).unwrap();
        name.set(self.data.name.clone());
        self.data.dirty = false;

        info!("Saving block data for: {}", *name);
    }

    /// Closes the current popup, if any.
    pub fn close_popup(&mut self) {
        self.data.popup = Popup::None;
    }

    /// Returns whether a popup is currently open.
    pub fn is_popup_open(&self) -> bool {
        self.data.popup != Popup::None
    }

    /// Returns the currently selected block.
    pub fn selected_block(&self) -> Entity {
        self.data.block_id
    }

    /// This function updates the face of a block in the block editor.
    pub fn update_block_face(&mut self, dir: FaceDirection, face: BlockFace) {
        let (_, _, _, mut shape) = self.blocks.get_mut(self.data.block_id).unwrap();

        let BlockShape::Cube {
            top,
            bottom,
            north,
            south,
            east,
            west,
            ..
        } = &mut *shape
        else {
            return;
        };

        match dir {
            FaceDirection::Up => *top = face,
            FaceDirection::Down => *bottom = face,
            FaceDirection::North => *north = face,
            FaceDirection::South => *south = face,
            FaceDirection::East => *east = face,
            FaceDirection::West => *west = face,
        }

        self.data.dirty = true;
    }

    /// This function renders the combo box for selecting a tileset, or an empty
    /// combo box if the block does not use a tileset.
    pub fn tileset_list_combobox(&mut self, ui: &mut egui::Ui) {
        let (_, _, _, shape) = self.blocks.get(self.data.block_id).unwrap();

        match shape {
            BlockShape::Cube { tileset, .. } => {
                let mut sel_tileset = tileset.clone();
                egui::ComboBox::from_label("tileset_list_select")
                    .selected_text(tileset)
                    .show_ui(ui, |ui| {
                        for (_, name, _, _) in self.tilesets.iter() {
                            let n = name.as_str().to_string();
                            ui.selectable_value(&mut sel_tileset, n, name.as_str());
                        }
                    });
            }
            _ => {
                egui::ComboBox::from_label("tileset_list_select")
                    .selected_text("")
                    .show_ui(ui, |_| {});
            }
        }
    }

    /// Returns the currently selected tileset image, if any.
    pub fn get_selected_tileset_image(&self) -> Option<&Handle<Image>> {
        let (_, _, _, shape) = self.blocks.get(self.data.block_id).unwrap();

        match shape {
            BlockShape::Cube { tileset, .. } => self
                .tilesets
                .iter()
                .find(|(_, name, _, _)| name.as_str() == tileset)
                .map(|(_, _, _, handle)| handle),
            _ => None,
        }
    }

    /// This function renders a list of tiles from the selected tileset, or an
    /// empty list if no tileset is selected.
    pub fn tile_list(
        &mut self,
        ui: &mut egui::Ui,
        tile_list_texture_id: Option<egui::TextureId>,
        selected_face: Option<FaceDirection>,
    ) {
        let Some(tile_list_texture_id) = tile_list_texture_id else {
            return;
        };

        let tile_size = 64.0;
        let columns = 6;

        egui::Grid::new("tileset_grid")
            .num_columns(columns)
            .spacing((2.0, 2.0))
            .min_col_width(tile_size)
            .max_col_width(tile_size)
            .min_row_height(tile_size)
            .striped(true)
            .show(ui, |ui| {
                let mut i = 0;
                for y in 0 .. TILESET_LENGTH as u8 {
                    for x in 0 .. TILESET_LENGTH as u8 {
                        if ui
                            .add(TileWidget {
                                texture: tile_list_texture_id,
                                tile_pos: TilePos::new(x, y),
                                size: tile_size,
                            })
                            .interact(egui::Sense::click())
                            .clicked()
                        {
                            if let Some(dir) = selected_face {
                                self.update_block_face(dir, BlockFace {
                                    tile: TilePos::new(x, y),
                                    ..default()
                                });
                            }
                        }

                        i += 1;
                        if i % columns == 0 {
                            ui.end_row();
                        }
                    }
                }
            });
    }
}

/// A small state machine that handles popups.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Popup {
    /// No popup is currently open.
    #[default]
    None,

    /// A popup that appears when the user tries to open another block without
    /// saving the current one.
    UnsavedChanges {
        /// The new block that the user is trying to open.
        new_block: Entity,
    },
}
