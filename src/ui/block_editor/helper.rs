//! This module handles storage and editing for the temporary block data that is
//! actively being edited.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy_egui::egui::{self, FontFamily, FontId, RichText};

use crate::blocks::tileset::Tileset;
use crate::blocks::{AIR_BLOCK_UUID, Block};

/// The data structure that holds the temporary block data that is being edited.
pub struct BlockEditData {
    /// The block entity that is being edited.
    pub entity: Entity,

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
            entity: Entity::PLACEHOLDER,
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
    blocks: Query<'w, 's, (Entity, &'static mut Name, &'static Block), Without<Tileset>>,
}

impl<'w, 's> BlockEditHelper<'w, 's> {
    /// This function is used to ensure that the block data is initialized. If
    /// the data is already initialized, this function does nothing.
    pub fn initialize(&mut self) {
        if self.data.entity != Entity::PLACEHOLDER {
            return;
        }

        let air = self
            .blocks
            .iter()
            .find(|(_, _, block)| block.uuid == AIR_BLOCK_UUID)
            .map(|(entity, _, _)| entity)
            .unwrap();

        self.select_block(air);
    }

    /// Adds a selectable list of all blocks to the UI.
    pub fn edit_block_list(&mut self, ui: &mut egui::Ui) {
        let block_list = self.blocks.iter().sort_by::<&Name>(|a, b| a.cmp(b));

        let mut sel_block = self.data.entity;
        for (block_id, name, _) in block_list {
            ui.selectable_value(
                &mut sel_block,
                block_id,
                RichText::new(name).monospace().size(20.0),
            );
        }

        if sel_block != self.data.entity {
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
        self.data.entity = block;
        self.data.dirty = false;

        let (_, name, _) = self.blocks.get(block).unwrap();
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

        let original_name = self.blocks.get(self.data.entity).unwrap().1;
        if self.data.name != original_name.as_str() {
            self.data.dirty = true;
        }
    }

    /// Saves the current block data.
    pub fn save_block(&mut self) {
        let (_, mut name, _) = self.blocks.get_mut(self.data.entity).unwrap();
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
        self.data.entity
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
