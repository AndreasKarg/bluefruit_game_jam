#![forbid(unsafe_code)]
#![cfg_attr(not(debug_assertions), deny(warnings))] // Forbid warnings in release builds

// ----------------------------------------------------------------------------
// When compiling for web:
use std::panic;

#[cfg(target_arch = "wasm32")]
use eframe::wasm_bindgen::{self, prelude::*};

use crate::{game::init_stuff, MyGame};

/// This is the entry-point for all the web-assembly.
/// This is called once from the HTML.
/// It loads the app, installs some callbacks, then returns.
/// You can add more callbacks like this if you want to call in to your code.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn start(canvas_id: &str) -> Result<(), eframe::wasm_bindgen::JsValue> {
    // web_sys::console::log_1(&"Start!".into());
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    let mut app = MyGame::default();
    init_stuff(&mut app.units);
    eframe::start_web(canvas_id, Box::new(app))
}
