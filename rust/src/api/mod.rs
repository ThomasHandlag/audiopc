pub mod source;
pub mod player;
pub mod renderer;
pub mod common;
pub mod enums;
pub mod error;

#[flutter_rust_bridge::frb(init)]
pub fn init_app() {
    // Default utilities - feel free to customize
    flutter_rust_bridge::setup_default_user_utils();
}