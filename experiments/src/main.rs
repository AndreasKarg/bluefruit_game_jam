use engine::bevy::prelude::*;

use crate::game::{gui, init_stuff, spawn_enemies, units_meet_enemies, EnemySpawner};

mod game;

fn main() {
    engine::run(MyGame);
}

struct MyGame;

impl Plugin for MyGame {
    fn build(&self, app: &mut AppBuilder) {
        app.add_startup_system(init_stuff.system())
            .init_resource::<EnemySpawner>()
            .add_system(gui.system())
            .add_system(units_meet_enemies.system())
            .add_system(spawn_enemies.system());
    }
}
