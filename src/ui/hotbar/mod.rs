//! This module adds functionality for creating the editor hotbar HUD element.

use bevy::asset::embedded_asset;
use bevy::prelude::*;
use bevy::render::view::RenderLayers;

use super::menu::MainMenuState;
use crate::map::blocks::{Block, RenderedBlock};

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
        app_.add_systems(
            OnEnter(MainMenuState::Editor),
            setup_hotbar.after(crate::map::setup),
        )
        .add_systems(OnExit(MainMenuState::Editor), cleanup_hotbar)
        .add_systems(
            Update,
            update_selected_index.run_if(in_state(MainMenuState::Editor)),
        );

        embedded_asset!(app_, "bg.png");
        embedded_asset!(app_, "selection.png");
    }
}

/// This is a marker component used to indicate that the entity is the root of
/// the hotbar.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Component)]
pub struct HotbarRoot;

/// This component is used to indicate that the entity is the hotbar selector.
#[derive(Debug, Default, Clone, Component)]
pub struct HotbarSelection {
    /// The currently selected index.
    pub index: usize,
}

/// This component is used to indicate that the entity is a hotbar slot.
#[derive(Debug, Default, Clone, Component)]
pub struct HotbarSlot {
    /// The index of the slot.
    pub index: usize,
}

/// This system is used to create the editor hotbar HUD element.
pub fn setup_hotbar(
    asset_server: Res<AssetServer>,
    blocks: Query<(Entity, &Name), With<Block>>,
    mut commands: Commands,
) {
    let hotbar_bg = asset_server.load(HOTBAR_BG_IMG);
    let hotbar_sel = asset_server.load(HOTBAR_SEL_IMG);

    let block_name: Name = "debug".into();
    let block_id = blocks
        .iter()
        .find(|(_, name)| **name == block_name)
        .map(|(entity, _)| entity)
        .unwrap();

    // let mut block_transform = Transform::from_translation(Vec3::splat(-0.5));
    let mut block_transform = Transform::from_rotation(Quat::from_euler(
        EulerRot::XYZ,
        45f32.to_radians(),
        45f32.to_radians(),
        180f32.to_radians(),
    ));
    block_transform.scale = Vec3::splat(HOTBAR_SIZE / 3f32.sqrt());

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
                    for i in 0 .. 10 {
                        parent
                            .spawn(ImageBundle {
                                style: Style {
                                    width: Val::Px(HOTBAR_SIZE),
                                    height: Val::Px(HOTBAR_SIZE),
                                    ..default()
                                },
                                image: hotbar_bg.clone().into(),
                                ..default()
                            })
                            .with_children(|parent| {
                                parent
                                    .spawn((
                                        HotbarSlot { index: i },
                                        ImageBundle {
                                            style: Style {
                                                position_type: PositionType::Absolute,
                                                width: Val::Percent(100.0),
                                                height: Val::Percent(100.0),
                                                ..default()
                                            },
                                            image: UiImage::solid_color(Color::NONE),
                                            ..default()
                                        },
                                    ))
                                    .with_children(|parent| {
                                        parent
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
                                            });
                                    });
                            });
                    }

                    parent.spawn((
                        HotbarSelection::default(),
                        ImageBundle {
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
                        },
                    ));
                });
        });
}

/// This system is used to cleanup the editor hotbar HUD element.
pub fn cleanup_hotbar(mut commands: Commands, hotbar_root: Query<Entity, With<HotbarRoot>>) {
    for entity in hotbar_root.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

/// This system listens for changes to the hotbar selection and updates the
/// position of the selector accordingly.
pub fn update_selected_index(
    mut selector: Query<(&mut Style, &HotbarSelection), Changed<HotbarSelection>>,
) {
    for (mut style, selection) in selector.iter_mut() {
        let left = selection.index as f32 * (HOTBAR_SIZE + HOTBAR_GAP);
        style.left = Val::Px(left);
    }
}
