use crate::game_of_life::RunStateSet;
use bevy::prelude::*;
use bevy::utils::HashSet;
use std::collections::HashMap;

pub(super) const CELL_SIZE: f32 = 10.;
const SPACING: f32 = 1.;

const INITIAL_DIM: i32 = 200;

pub(super) struct CellPlugin;

impl Plugin for CellPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CellMap>()
            .init_resource::<CellsToFlip>()
            .init_resource::<DeadNextToAlive>()
            .register_type::<GameCell>()
            .add_systems(Startup, (init_cells, configure_cells).chain())
            .add_systems(Update, (tick_cells, flip_cells).chain().in_set(RunStateSet));
    }
}

#[derive(Resource)]
pub(super) struct CellColors {
    alive_color: Handle<ColorMaterial>,
    dead_color: Handle<ColorMaterial>,
}

#[derive(Component, Reflect)]
pub(super) struct GameCell {
    neighbors: [Option<Entity>; 8],
    pub(super) alive: bool,
    pub(super) cell_pos: IVec2,
}

#[derive(Component)]
#[component(storage = "SparseSet")]
pub(super) struct AliveCell;

impl GameCell {
    fn get_neighbor_index(mut offset: IVec2) -> usize {
        offset += 1;
        let mut index = (offset.y * 3) + offset.x;
        if index == 8 {
            index = 4;
        }

        index as usize
    }

    fn get_neighbor(&self, offset: IVec2) -> Option<Entity> {
        self.neighbors[Self::get_neighbor_index(offset)]
    }

    fn set_neighbor(&mut self, offset: IVec2, entity: Option<Entity>) {
        self.neighbors[Self::get_neighbor_index(offset)] = entity;
    }

    pub(super) fn set_alive(
        &mut self,
        alive: bool,
        mesh_material: &mut MeshMaterial2d<ColorMaterial>,
        cell_colors: &CellColors,
        mut entity_commands: EntityCommands,
    ) {
        self.alive = alive;

        mesh_material.0 = if self.alive {
            entity_commands.insert(AliveCell);
            cell_colors.alive_color.clone()
        } else {
            entity_commands.remove::<AliveCell>();
            cell_colors.dead_color.clone()
        }
    }
}

#[derive(Component)]
struct ConfigureNeighbors;

#[derive(Resource, Default, Deref, DerefMut)]
pub(super) struct CellMap(HashMap<IVec2, Entity>);

#[derive(Resource, Default, Deref, DerefMut)]
pub(super) struct CellsToFlip(Vec<Entity>);

#[derive(Resource, Default, Deref, DerefMut)]
pub(super) struct DeadNextToAlive(HashSet<Entity>);

fn init_cells(
    mut commands: Commands,
    mut cell_map: ResMut<CellMap>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let alive_color = materials.add(Color::WHITE);
    let dead_color = materials.add(Color::BLACK);

    commands.insert_resource(CellColors {
        alive_color,
        dead_color: dead_color.clone(),
    });

    for x in -INITIAL_DIM..=INITIAL_DIM {
        for y in -INITIAL_DIM..=INITIAL_DIM {
            let cell_pos = IVec2::new(x, y);
            let material_handle = dead_color.clone();

            let entity = commands
                .spawn((
                    GameCell {
                        neighbors: [None; 8],
                        alive: false,
                        cell_pos,
                    },
                    Mesh2d(meshes.add(Rectangle::new(CELL_SIZE, CELL_SIZE))),
                    MeshMaterial2d(material_handle),
                    Transform::from_translation(
                        cell_pos.extend(0).as_vec3() * (CELL_SIZE * SPACING),
                    ),
                    ConfigureNeighbors,
                ))
                .id();

            cell_map.insert(cell_pos, entity);
        }
    }
}

fn tick_cells(
    alive_cells: Query<(Entity, &GameCell), With<AliveCell>>,
    cell_query: Query<&GameCell>,
    mut dead_next_to_alive: ResMut<DeadNextToAlive>,
    mut cells_to_flip: ResMut<CellsToFlip>,
) {
    for (entity, cell) in alive_cells.iter() {
        let mut around_alive = 0;
        for neighbor_entity in cell.neighbors {
            if let Some(neighbor_entity) = neighbor_entity {
                let cell = cell_query.get(neighbor_entity).unwrap();
                if cell.alive {
                    around_alive += 1;
                } else {
                    dead_next_to_alive.insert(neighbor_entity);
                }
            }
        }

        if around_alive < 2 || around_alive > 3 {
            cells_to_flip.push(entity);
        }
    }

    for dead_cell_entity in dead_next_to_alive.drain() {
        let cell = cell_query.get(dead_cell_entity).unwrap();
        let mut around_alive = 0;
        for neighbor_entity in cell.neighbors {
            if let Some(neighbor_entity) = neighbor_entity {
                let cell = cell_query.get(neighbor_entity).unwrap();
                if cell.alive {
                    around_alive += 1;
                }
            }
        }

        if around_alive == 3 {
            cells_to_flip.push(dead_cell_entity);
        }
    }
}

fn flip_cells(
    mut commands: Commands,
    mut cell_query: Query<(&mut GameCell, &mut MeshMaterial2d<ColorMaterial>)>,
    cell_colors: Res<CellColors>,
    mut cells_to_flip: ResMut<CellsToFlip>,
) {
    let cell_len = cells_to_flip.len();
    for entity in cells_to_flip.drain(0..cell_len) {
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

fn configure_cells(
    mut commands: Commands,
    mut configure_query: Query<(Entity, &mut GameCell), Added<ConfigureNeighbors>>,
    cell_map: Res<CellMap>,
) {
    for (entity, mut cell) in configure_query.iter_mut() {
        for x in -1..=1 {
            for y in -1..=1 {
                let offset = IVec2::new(x, y);
                let cell_pos = cell.cell_pos;
                cell.set_neighbor(offset, cell_map.get(&(offset + cell_pos)).cloned())
            }
        }
        commands.entity(entity).remove::<ConfigureNeighbors>();
    }
}
