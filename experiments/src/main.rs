use engine::bevy::prelude::*;

use crate::game::{gui, init_stuff};

mod game;

fn main() {
    engine::run(MyGame);
}

struct MyGame;

impl Plugin for MyGame {
    fn build(&self, app: &mut AppBuilder) {
        app.add_startup_system(init_stuff.system())
            .add_system(gui.system());
    }
}
