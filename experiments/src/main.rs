use engine::bevy::prelude::*;

use crate::game::{
    gui, init_stuff, spawn_enemies, ticker, units_meet_enemies, EnemySpawner, GameOver,
    ParkingSpace, TokenPool,
};

mod game;
mod todo;

fn main() {
    engine::run(MyGame);
}

struct MyGame;

impl Plugin for MyGame {
    fn build(&self, app: &mut AppBuilder) {
        app.add_startup_system(init_stuff.system())
            .init_resource::<EnemySpawner>()
            .insert_resource(TokenPool::<ParkingSpace>::new(2))
            .add_event::<GameOver>()
            .add_system(gui.system())
            .add_system(ticker.system())
            .add_system(units_meet_enemies.system())
            .add_system(spawn_enemies.system());
    }
}
