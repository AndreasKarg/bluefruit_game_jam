// use engine::eframe::{egui::CtxRef, epi, epi::Frame};

use eframe::{egui::CtxRef, epi, epi::Frame};

use crate::{
    game::{
        gui, init_stuff, repair_tick, spawn_enemies, ticker, units_meet_enemies, Enemy,
        EnemySpawner, GameState, ParkingSpace, PlayTime, TokenPool, Unit, UnitBundle,
    },
    helpers::Time,
};

mod game;
mod helpers;
mod todo;
mod wasm_startup;

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    engine::run(MyGame, "Fruitopian Defender");
}

#[derive(Default)]
pub struct MyGame {
    enemy_spawner: EnemySpawner,
    play_time: PlayTime,
    parking_spaces: TokenPool<ParkingSpace>,
    game_state: GameState,
    time: Time,
    units: Vec<UnitBundle>,
    enemies: Vec<Enemy>,
}

impl epi::App for MyGame {
    fn update(&mut self, ctx: &CtxRef, frame: &mut Frame<'_>) {
        ctx.request_repaint();
        // web_sys::console::log_1(&format!("GameState: {:#?}", self.game_state).into());
        if self.game_state == GameState::Running {
            ticker(
                &self.time,
                self.units.as_mut_slice(),
                self.enemies.as_mut_slice(),
                &mut self.game_state,
                &mut self.play_time,
            );

            units_meet_enemies(&mut self.units, &mut self.enemies);
            spawn_enemies(&mut self.enemy_spawner, &self.time, &mut self.enemies);
            repair_tick(&self.time, self.units.as_mut_slice());
        }

        gui(
            ctx,
            self.units.as_mut_slice(),
            self.enemies.as_mut_slice(),
            &mut self.parking_spaces,
            &self.game_state,
            &self.play_time,
        );

        self.time.tick();
    }

    fn name(&self) -> &str {
        "Fruitopian Defender"
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
