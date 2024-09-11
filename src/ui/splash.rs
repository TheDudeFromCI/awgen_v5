//! This module contains the implementation of the splash screen UI plugin.

use bevy::prelude::*;

use super::menu::MainMenuState;

/// The asset path to the Wraithaven Games splash screen icon.
const WHG_SPLASH_ICON: &str = "ui/splash/whg.png";

/// The maximum size of the splash screen icon.
const SPLASH_MAX_SIZE: f32 = 1024.0;

/// The plugin responsible for managing the splash screen UI.
pub struct SplashPlugin;
impl Plugin for SplashPlugin {
    fn build(&self, app_: &mut App) {
        app_.add_systems(Startup, build_splash)
            .add_systems(OnExit(MainMenuState::Splash), dispose_splash)
            .add_systems(
                Update,
                update_splash.run_if(in_state(MainMenuState::Splash)),
            );
    }
}

/// This is a marker component that indicates the root of the splash screen.
#[derive(Debug, Component)]
struct SplashScreenRoot;

/// This is a marker component that indicates the splash screen icon.
#[derive(Debug, Component)]
struct SplashIcon;

/// Builds the splash screen.
fn build_splash(asset_server: Res<AssetServer>, mut commands: Commands) {
    commands
        .spawn((
            SplashScreenRoot,
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                background_color: Color::BLACK.into(),
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                SplashIcon,
                ImageBundle {
                    style: Style {
                        width: Val::Auto,
                        height: Val::Percent(75.0),
                        max_height: Val::Px(SPLASH_MAX_SIZE),
                        aspect_ratio: Some(1.0),
                        margin: UiRect::all(Val::Auto),
                        ..default()
                    },
                    image: UiImage::new(asset_server.load(WHG_SPLASH_ICON))
                        .with_color(Color::WHITE),
                    ..default()
                },
            ));
        });
}

/// Updates the splash screen animation.
fn update_splash(
    time: Res<Time>,
    mut icon: Query<&mut UiImage, With<SplashIcon>>,
    mut next_state: ResMut<NextState<MainMenuState>>,
) {
    /// The time in seconds to wait before fading in the splash icon.
    const INIT_TIME: f32 = 1.0;

    /// The time in seconds to fade in/out the splash icon.
    const FADE_TIME: f32 = 1.0;

    /// The time in seconds to hold the splash icon at full opacity.
    const HOLD_TIME: f32 = 1.5;

    /// The time in seconds to wait before transitioning to the main menu.
    const END_TIME: f32 = 1.0;

    let seconds = time.elapsed_seconds();

    let alpha = if seconds < INIT_TIME {
        0.0
    } else if seconds < INIT_TIME + FADE_TIME {
        (seconds - INIT_TIME) / FADE_TIME
    } else if seconds < INIT_TIME + FADE_TIME + HOLD_TIME {
        1.0
    } else if seconds < INIT_TIME + FADE_TIME + HOLD_TIME + FADE_TIME {
        1.0 - (seconds - INIT_TIME - FADE_TIME - HOLD_TIME) / FADE_TIME
    } else {
        0.0
    };

    for mut image in icon.iter_mut() {
        image.color = Color::srgba(1.0, 1.0, 1.0, alpha);
    }

    if seconds >= INIT_TIME + FADE_TIME + HOLD_TIME + FADE_TIME + END_TIME {
        next_state.set(MainMenuState::Editor);
    }
}

/// Disposes the splash screen.
fn dispose_splash(mut commands: Commands, query: Query<Entity, With<SplashScreenRoot>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
