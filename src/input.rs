use crate::cell::{
    AliveCell, CellColors, CellMap, CellsToFlip, DeadNextToAlive, GameCell, CELL_SIZE,
};
use crate::game_of_life::{GameCamera, IsPlaying};
use bevy::prelude::*;
use bevy::utils::AHasher;
use bevy::window::PrimaryWindow;
use std::hash::{Hash, Hasher};

pub(super) struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (mouse_click, key_input, randomize_cells));
    }
}

fn mouse_click(
    mut commands: Commands,
    window_query: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<GameCamera>>,
    mut cell_query: Query<(&mut GameCell, &mut MeshMaterial2d<ColorMaterial>)>,
    cell_map: Res<CellMap>,
    cell_colors: Res<CellColors>,
    mouse: Res<ButtonInput<MouseButton>>,
) {
    if mouse.just_pressed(MouseButton::Left) {
        let window = window_query.single();
        let (camera, global_transform) = camera_query.single();
        if let Some(mouse_pos) = window.cursor_position() {
            let mouse_world_pos = camera
                .viewport_to_world_2d(global_transform, mouse_pos)
                .unwrap();

            let cell_pos = (mouse_world_pos / CELL_SIZE).round().as_ivec2();

            if let Some(entity) = cell_map.get(&cell_pos) {
                let entity = *entity;
                let (mut cell, mut mesh_material) = cell_query.get_mut(entity).unwrap();

                let is_alive = cell.alive;
                cell.set_alive(
                    !is_alive,
                    &mut mesh_material,
                    &cell_colors,
                    commands.entity(entity),
                );
            }
        }
    }
}

fn key_input(
    mut commands: Commands,
    mut alive_query: Query<
        (Entity, &mut GameCell, &mut MeshMaterial2d<ColorMaterial>),
        With<AliveCell>,
    >,
    key: Res<ButtonInput<KeyCode>>,
    cell_colors: Res<CellColors>,

    mut is_playing: ResMut<IsPlaying>,
    mut cells_to_flip: ResMut<CellsToFlip>,
    mut dead_next_to_alive: ResMut<DeadNextToAlive>,
) {
    if key.just_pressed(KeyCode::Space) {
        is_playing.0 = !is_playing.0;
    }

    if key.just_pressed(KeyCode::KeyR) {
        for (entity, mut cell, mut mesh_material) in alive_query.iter_mut() {
            cell.set_alive(
                false,
                &mut mesh_material,
                &cell_colors,
                commands.entity(entity),
            );
        }

        cells_to_flip.clear();
        dead_next_to_alive.clear();
        is_playing.0 = false;
    }
}

fn randomize_cells(
    mut commands: Commands,
    mut cell_query: Query<(Entity, &mut GameCell, &mut MeshMaterial2d<ColorMaterial>)>,
    time: Res<Time>,
    key: Res<ButtonInput<KeyCode>>,
    cell_colors: Res<CellColors>,
    mut is_playing: ResMut<IsPlaying>,
    mut cells_to_flip: ResMut<CellsToFlip>,
    mut dead_next_to_alive: ResMut<DeadNextToAlive>,
) {
    if key.just_pressed(KeyCode::KeyT) {
        let mut hasher = AHasher::default();
        time.elapsed().as_millis().hash(&mut hasher);

        for (entity, mut cell, mut mesh_material) in cell_query.iter_mut() {
            cell.cell_pos.hash(&mut hasher);
            let val = hasher.finish() as i64;
            let alive = if val > 0 { true } else { false };
            cell.set_alive(
                alive,
                &mut mesh_material,
                &cell_colors,
                commands.entity(entity),
            );
        }

        cells_to_flip.clear();
        dead_next_to_alive.clear();
        is_playing.0 = false;
    }
}
