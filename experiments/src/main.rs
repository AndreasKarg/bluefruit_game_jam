use engine::bevy::prelude::*;

use crate::game::{ui_example, UiState};

mod game;

fn main() {
    engine::run(MyGame);
}

struct MyGame;

impl Plugin for MyGame {
    fn build(&self, app: &mut AppBuilder) {
        app.init_resource::<UiState>()
            .add_system(ui_example.system());
    }
}
