use bevy::prelude::*;
use bevy_egui::{egui, EguiContext, EguiPlugin, EguiSettings};

pub extern crate bevy;
pub extern crate bevy_egui;

pub fn run<G: Plugin>(game: G) {
    App::build()
        .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
        .insert_resource(Msaa { samples: 4 })
        .add_plugins(DefaultPlugins)
        .add_plugin(EguiPlugin)
        .add_plugin(game)
        .add_startup_system(load_assets.system())
        .add_system(update_ui_scale_factor.system())
        .run();
}

fn load_assets(mut egui_context: ResMut<EguiContext>, assets: Res<AssetServer>) {
    // let texture_handle = assets.load("icon.png");
    // egui_context.set_egui_texture(BEVY_TEXTURE_ID, texture_handle);
}

fn update_ui_scale_factor(mut egui_settings: ResMut<EguiSettings>, windows: Res<Windows>) {
    if let Some(window) = windows.get_primary() {
        egui_settings.scale_factor = 1.5 / window.scale_factor();
    }
}
