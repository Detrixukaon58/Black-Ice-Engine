
use std::{sync::Arc, thread::JoinHandle};
use parking_lot::*;
use crate::common::{engine::gamesys::*, components::entity::entity_system::*};

pub enum InputValue {

}

pub struct Input {

}

pub struct InputSystem {
    input_buffer: Vec<InputValue>,
    input_event_handlers: Vec<Arc<Mutex<Input>>>
}

impl InputSystem {

    pub unsafe fn input(this: Arc<Mutex<Self>>) -> JoinHandle<i32> {

        std::thread::Builder::new().name("Input".to_string()).spawn(|| {InputSystem::process(this)}).unwrap()

    }

    pub unsafe fn process(p_this: Arc<Mutex<Self>>) -> i32 {


        while !Game::isExit() {

            


        }
        

        0
    }

}