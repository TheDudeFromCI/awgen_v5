//! This module adds functionality for creating the editor hotbar HUD element.

use bevy::asset::embedded_asset;
use bevy::prelude::*;
use bevy::render::view::RenderLayers;
use bevy_mod_picking::PickableBundle;
use bevy_mod_picking::events::{Click, Pointer};
use bevy_mod_picking::prelude::{On, Pickable};
use resource::{Hotbar, HotbarSlotData};

use super::menu::MainMenuState;
use crate::blocks::RenderedBlock;
use crate::tools::Tool;

pub mod resource;

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
                setup_hotbar
                    .before_ignore_deferred(crate::map::editor::startup::prepare_map_editor),
            )
            .add_systems(OnExit(MainMenuState::Editor), cleanup_hotbar)
            .add_systems(
                Update,
                (
                    select_slot_with_numkeys.run_if(in_state(MainMenuState::Editor)),
                    update_selected_index
                        .run_if(in_state(MainMenuState::Editor))
                        .run_if(resource_changed::<Hotbar>)
                        .after_ignore_deferred(select_slot_with_numkeys),
                    update_slot_visuals
                        .run_if(in_state(MainMenuState::Editor))
                        .run_if(resource_changed::<Hotbar>)
                        .after_ignore_deferred(update_selected_index),
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

/// This system is used to create the editor hotbar HUD element.
pub fn setup_hotbar(
    asset_server: Res<AssetServer>,
    mut hotbar: ResMut<Hotbar>,
    mut commands: Commands,
) {
    let hotbar_bg = asset_server.load(HOTBAR_BG_IMG);
    let hotbar_sel = asset_server.load(HOTBAR_SEL_IMG);

    hotbar.activate();

    commands
        .spawn((
            HotbarRoot,
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                ..default()
            },
            Pickable::IGNORE,
        ))
        .with_children(|parent| {
            parent
                .spawn(NodeBundle {
                    style: Style {
                        margin: UiRect {
                            bottom: Val::Px(2.0),
                            left: Val::Auto,
                            right: Val::Auto,
                            top: Val::Auto,
                        },
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(HOTBAR_GAP),
                        ..default()
                    },
                    ..default()
                })
                .with_children(|parent| {
                    for slot_index in 0 .. 10 {
                        parent
                            .spawn((
                                ImageBundle {
                                    style: Style {
                                        width: Val::Px(HOTBAR_SIZE),
                                        height: Val::Px(HOTBAR_SIZE),
                                        ..default()
                                    },
                                    image: hotbar_bg.clone().into(),
                                    ..default()
                                },
                                On::<Pointer<Click>>::run(move |mut hotbar: ResMut<Hotbar>| {
                                    hotbar.select_slot(slot_index);
                                }),
                            ))
                            .with_children(|parent| {
                                let slot_id = parent
                                    .spawn((HotbarSlot, ImageBundle {
                                        style: Style {
                                            position_type: PositionType::Absolute,
                                            width: Val::Percent(100.0),
                                            height: Val::Percent(100.0),
                                            ..default()
                                        },
                                        image: UiImage::solid_color(Color::NONE),
                                        ..default()
                                    }))
                                    .id();
                                hotbar.insert_slot(slot_id);
                            });
                    }

                    parent.spawn((HotbarSelector, ImageBundle {
                        style: Style {
                            position_type: PositionType::Absolute,
                            width: Val::Px(HOTBAR_SIZE),
                            height: Val::Px(HOTBAR_SIZE),
                            top: Val::Px(0.0),
                            left: Val::Px(0.0),
                            ..default()
                        },
                        image: hotbar_sel.into(),
                        ..default()
                    }));
                });
        });
}

/// This system is used to cleanup the editor hotbar HUD element.
pub fn cleanup_hotbar(
    mut hotbar: ResMut<Hotbar>,
    hotbar_root: Query<Entity, With<HotbarRoot>>,
    mut commands: Commands,
) {
    for entity in hotbar_root.iter() {
        commands.entity(entity).despawn_recursive();
    }
    hotbar.deactivate();
}

/// This system listens for changes to the hotbar selection and updates the
/// position of the selector accordingly.
pub fn update_selected_index(
    hotbar: Res<Hotbar>,
    mut selector: Query<&mut Style, With<HotbarSelector>>,
) {
    if !hotbar.is_active() {
        return;
    }

    for mut style in selector.iter_mut() {
        let left = hotbar.get_selected_index() as f32 * (HOTBAR_SIZE + HOTBAR_GAP);
        style.left = Val::Px(left);
    }
}

/// This system listens for number key presses and selects the corresponding
/// slot if it exists.
pub fn select_slot_with_numkeys(mut hotbar: ResMut<Hotbar>, input: Res<ButtonInput<KeyCode>>) {
    /// The key codes for the first 10 keyboard number keys.
    const KEYS: [KeyCode; 10] = [
        KeyCode::Digit1,
        KeyCode::Digit2,
        KeyCode::Digit3,
        KeyCode::Digit4,
        KeyCode::Digit5,
        KeyCode::Digit6,
        KeyCode::Digit7,
        KeyCode::Digit8,
        KeyCode::Digit9,
        KeyCode::Digit0,
    ];

    let slots = usize::min(hotbar.slot_count(), KEYS.len());

    for (i, key) in KEYS.iter().enumerate().take(slots) {
        if input.just_pressed(*key) {
            hotbar.select_slot(i);
            break;
        }
    }
}

/// This system updates the visuals of the hotbar slots based on the data in the
/// hotbar resource.
pub fn update_slot_visuals(
    mut hotbar: ResMut<Hotbar>,
    tools: Query<&UiImage, (With<Tool>, Without<HotbarSlot>)>,
    mut slots: Query<&mut UiImage, With<HotbarSlot>>,
    mut commands: Commands,
) {
    for i in 0 .. hotbar.slot_count() {
        if !hotbar.is_dirty(i) {
            continue;
        }

        let slot_id = hotbar.get_slot_entity(i);
        commands.entity(slot_id).despawn_descendants();

        let mut slot_icon = slots.get_mut(slot_id).unwrap();

        match hotbar.get_slot(i) {
            HotbarSlotData::Empty => {
                *slot_icon = UiImage::default();
            }
            HotbarSlotData::Tool(tool_id) => {
                let tool_icon = tools.get(tool_id).unwrap();
                *slot_icon = tool_icon.clone();
            }
            HotbarSlotData::Block(block_id) => {
                *slot_icon = UiImage::default();

                let mut block_transform = Transform::from_rotation(Quat::from_euler(
                    EulerRot::XYZ,
                    45f32.to_radians(),
                    45f32.to_radians(),
                    180f32.to_radians(),
                ));
                block_transform.scale = Vec3::splat(HOTBAR_SIZE / 3f32.sqrt());

                commands
                    .spawn(SpatialBundle {
                        transform: Transform::from_translation(Vec3::new(
                            0.0,
                            HOTBAR_SIZE / 2.0,
                            0.0,
                        )),
                        ..default()
                    })
                    .with_children(|parent| {
                        parent.spawn((
                            RenderLayers::layer(1),
                            RenderedBlock { block: block_id },
                            PbrBundle {
                                transform: block_transform,
                                ..default()
                            },
                        ));
                    })
                    .set_parent(slot_id);
            }
        }
    }

    hotbar.mark_clean();
}
