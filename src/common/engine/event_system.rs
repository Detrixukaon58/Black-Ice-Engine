

use std::{sync::Arc, thread::JoinHandle, collections::HashMap};
use parking_lot::*;
use crate::common::*;

#[derive(Clone)]
pub enum Event {

}

pub enum EventData {

}

#[derive(Clone)]
pub struct EventHandler {

    event_type: Event,
    event_data: HashMap<String, Arc<EventData>>

}

pub struct EventSystem {
    events: Vec<Event>,
    event_handlers: Vec<Arc<Mutex<EventHandler>>>
}

impl EventSystem {

    pub unsafe fn init(this:Arc<Mutex<Self>>) -> JoinHandle<i32> {
        std::thread::Builder::new().name("Event".to_string()).spawn(|| {EventSystem::process(this)}).unwrap()
    }

    pub unsafe fn process(p_this: Arc<Mutex<Self>>) -> i32 {

        while !Game::isExit() {

            

        }

        0
    }
}