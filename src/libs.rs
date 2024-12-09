use std::sync::Arc;

use black_ice::common::engine::gamesys::*;
use once_cell::sync::Lazy;
use parking_lot::Mutex;

extern crate shaderc;

pub mod black_ice {
    pub mod common;
}

#[no_mangle]
pub unsafe fn init_game_env() {
    ENV = Lazy::new( || {Some(Arc::new(Mutex::new(Env::new_sdl())))});
    Env::init();
}