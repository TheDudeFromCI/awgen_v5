//! This module implements the [`BlockFaceGizmo`] component, which is used to
//! show what block face the user is currently hovering over.

use std::f32::consts::TAU;

use bevy::pbr::{NotShadowCaster, NotShadowReceiver};
use bevy::prelude::*;
use bevy_mod_picking::prelude::Pickable;

use super::cursor::CursorRaycast;

/// The asset path to the Wraithaven Games splash screen icon.
pub const GIZMO_FACE_MODEL: &str = "embedded://awgen/gizmos/block_face.glb";

/// A marker component that indicates the entity is a block face gizmo model.
#[derive(Debug, Default, Component)]
pub struct BlockFaceGizmo;

/// The inner part of the block face gizmo.
#[derive(Debug, Default, Component)]
pub struct BlockFaceGizmoInner;

/// This system updates the position of the block face gizmo to match the cursor
/// target.
pub fn update_block_face_gizmo(
    cursor: Res<CursorRaycast>,
    mut gizmo: Query<(&mut Transform, &mut Visibility), With<BlockFaceGizmo>>,
) {
    let Ok((mut transform, mut visibility)) = gizmo.get_single_mut() else {
        return;
    };

    let Some(hit) = &cursor.block else {
        *visibility = Visibility::Hidden;
        return;
    };

    *visibility = Visibility::Inherited;
    transform.translation = hit.block.as_vec3() + Vec3::ONE * 0.5;
    transform.rotation = hit.face.rotation_quat();
}

/// This system creates the block face gizmo model.
pub fn build_block_face_gizmo(
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
) {
    let inner_mesh = asset_server.load(
        GltfAssetLabel::Primitive {
            mesh: 0,
            primitive: 0,
        }
        .from_asset(GIZMO_FACE_MODEL),
    );

    let outer_mesh = inner_mesh.clone();

    let inner_material = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 1.0, 1.0),
        unlit: true,
        ..default()
    });

    let outer_material = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 1.0, 1.0),
        unlit: true,
        alpha_mode: AlphaMode::Blend,

        ..default()
    });

    commands
        .spawn((
            BlockFaceGizmo,
            NotShadowCaster,
            NotShadowReceiver,
            Pickable::IGNORE,
            PbrBundle {
                mesh: outer_mesh,
                material: outer_material,
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                BlockFaceGizmoInner,
                NotShadowCaster,
                NotShadowReceiver,
                Pickable::IGNORE,
                PbrBundle {
                    mesh: inner_mesh,
                    material: inner_material,
                    ..default()
                },
            ));
        });
}

/// This system animates the block face gizmo.
#[allow(clippy::type_complexity)]
pub fn animate_block_face_gizmo(
    time: Res<Time>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut outer_gizmo: Query<
        (&mut Transform, &Handle<StandardMaterial>),
        (With<BlockFaceGizmo>, Without<BlockFaceGizmoInner>),
    >,
    mut inner_gizmo: Query<&mut Transform, (With<BlockFaceGizmoInner>, Without<BlockFaceGizmo>)>,
) {
    let Ok((mut outer_transform, outer_material_handle)) = outer_gizmo.get_single_mut() else {
        return;
    };

    let Ok(mut inner_transform) = inner_gizmo.get_single_mut() else {
        return;
    };

    let Some(outer_material) = materials.get_mut(outer_material_handle) else {
        return;
    };

    let pulse_speed = 5.0;
    let inner_pulse_size = 0.1;
    let outer_pulse_size = 0.3;

    let time = time.elapsed_seconds() * pulse_speed;

    let inner_scale = 1.0 + time.sin() * inner_pulse_size;
    inner_transform.scale = Vec3::new(inner_scale, inner_scale, 1.0);

    let outer_time = (time % TAU) * 0.5;
    let outer_scale = 1.0 + outer_time.sin() * outer_pulse_size;
    outer_transform.scale = Vec3::new(outer_scale, outer_scale, 1.0);

    let outer_alpha = outer_time.cos().clamp(0.0, 1.0);
    outer_material.base_color = Color::srgba(1.0, 1.0, 1.0, outer_alpha);
}
