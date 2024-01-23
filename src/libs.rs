use black_ice::common::engine::gamesys::*;
use once_cell::sync::Lazy;

extern crate shaderc;

pub mod black_ice {
    pub mod common;
}

#[no_mangle]
pub unsafe fn init_game_env() {
    ENV = Lazy::new( || {Some(Env::new_sdl())});
    Env::get_env().init();
}