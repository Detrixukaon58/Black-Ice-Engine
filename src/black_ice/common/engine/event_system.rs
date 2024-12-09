#![allow(unused)]

use std::{sync::Arc, thread::JoinHandle, collections::HashMap};
use colored::Colorize;
use parking_lot::*;
use crate::black_ice::common::{*, engine::pipeline::RenderPipelineSystem, Env};
use sdl2::{*, sys::*, mouse::MouseButton};

// This needs to handle all types of events depending on what system it is currently being built for

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
    event_handlers: Vec<Arc<Mutex<EventHandler>>>,
    event_pump: Vec<Arc<sdl2::event::Event>>,
    ready: bool,
}

unsafe impl Send for EventSystem {}

impl EventSystem {

    pub fn init(this:Arc<Mutex<Self>>) -> i32 {
        unsafe {Self::process(this)}
    }

    pub unsafe fn process(p_this: Arc<Mutex<Self>>) -> i32 {

        loop {
            let mut this = p_this.lock();
            if this.ready {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(5));
        }

        while !Env::isExit() {
            let mut this = p_this.lock();
            let mut event_pump = this.event_pump.clone();
            this.event_pump.clear();
            drop(this);
            while let Some(event) = event_pump.pop() {
                match *event {
                    event::Event::Quit {..} =>  {
                        unsafe{Env::set_status(StatusCode::CLOSE);}
                        //println!("Close sent");
                        
                    }
                    event::Event::Window { timestamp, window_id, win_event } => {
                        match win_event {
                            event::WindowEvent::Resized(x, y) => {
                                
                            }
                            _ => {}
                        }
                    }
                    _ => continue
                }
            }
        }
            
        
        

        0
    }

    pub fn new() -> Self {
        unsafe {
            
            Self { 
                events: Vec::new(), 
                event_handlers: Vec::new(), 
                event_pump: Vec::new(),
                ready: false
            }
        }
    }

    pub fn start(p_this: Arc<Mutex<Self>>) {
        let mut this = p_this.lock();
        //println!("{}", "Starting Event Thread!!".yellow());
        this.ready = true;
    }

    pub fn send_events(&mut self, events: &mut Vec<Arc<sdl2::event::Event>>)
    {
        self.event_pump.append(events);
    }

}