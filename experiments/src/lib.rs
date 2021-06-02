// use engine::eframe::{egui::CtxRef, epi, epi::Frame};

use eframe::{egui::CtxRef, epi, epi::Frame};

use crate::game::{
    gui, init_stuff, repair_tick, spawn_enemies, ticker, units_meet_enemies, EnemySpawner,
    GameState, ParkingSpace, PlayTime, TokenPool,
};

mod game;
mod helpers;
mod todo;
mod wasm_startup;

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    engine::run(MyGame, "Fruitopian Defender");
}

pub struct MyGame;

impl Default for MyGame {
    fn default() -> Self {
        Self {}
    }
}

impl epi::App for MyGame {
    fn update(&mut self, ctx: &CtxRef, frame: &mut Frame<'_>) {
        todo!()
    }

    fn name(&self) -> &str {
        todo!()
    }
}

// impl Plugin for MyGame {
//     fn build(&self, app: &mut AppBuilder) {
//         app.add_startup_system(init_stuff.system())
//             .init_resource::<EnemySpawner>()
//             .init_resource::<PlayTime>()
//             .insert_resource(TokenPool::<ParkingSpace>::new(3))
//             .add_state(GameState::Running)
//             .add_system(gui.system())
//             .add_system_set(
//                 SystemSet::on_update(GameState::Running)
//                     .with_system(ticker.system())
//                     .with_system(units_meet_enemies.system())
//                     .with_system(spawn_enemies.system())
//                     .with_system(repair_tick.system()),
//             );
//     }
// }
