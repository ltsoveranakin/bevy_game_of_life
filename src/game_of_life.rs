use crate::cell::CellPlugin;
use crate::input::InputPlugin;
use bevy::prelude::*;
use bevy::time::common_conditions::on_timer;
use bevy_editor_pls::EditorPlugin;
use std::time::Duration;

pub(super) struct GameOfLifePlugin;

impl Plugin for GameOfLifePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(DefaultPlugins)
            .add_plugins(EditorPlugin::new())
            .add_plugins((CellPlugin, InputPlugin))
            .init_resource::<IsPlaying>()
            .register_type::<IsPlaying>()
            .configure_sets(
                Update,
                RunStateSet
                    .run_if(resource_equals(IsPlaying(true)))
                    .run_if(on_timer(Duration::from_millis(100))),
            )
            .add_systems(Startup, spawn_camera);
    }
}

#[derive(Resource, Reflect, PartialEq, Default)]
#[reflect(Resource)]
pub(super) struct IsPlaying(pub(super) bool);

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub(super) struct RunStateSet;

#[derive(Component)]
pub(super) struct GameCamera;

fn spawn_camera(mut commands: Commands) {
    commands.spawn((Camera2d, GameCamera));
}
