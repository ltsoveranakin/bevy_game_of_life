mod cell;
mod game_of_life;
mod input;

use crate::game_of_life::GameOfLifePlugin;
use bevy::prelude::*;

fn main() {
    let mut app = App::new();
    app.add_plugins(GameOfLifePlugin).run();
}
