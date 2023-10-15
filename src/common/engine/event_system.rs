#![allow(unused)]

use std::{sync::Arc, thread::JoinHandle, collections::HashMap};
use colored::Colorize;
use parking_lot::*;
use crate::common::{*, engine::pipeline::RenderPipelineSystem};
use sdl2::{*, sys::*, mouse::MouseButton};

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

        while !Game::isExit() {
            let mut this = p_this.lock();
            let mut event_pump = this.event_pump.clone();
            this.event_pump.clear();
            drop(this);
            while let Some(event) = event_pump.pop() {
                match *event {
                    event::Event::Quit {..} =>  {
                        unsafe{Game::set_status(StatusCode::CLOSE);}
                        println!("Close sent");
                        
                    }
                    event::Event::Window { timestamp, window_id, win_event } => {
                        match win_event {
                            event::WindowEvent::Resized(x, y) => {
                                GAME.window_x = x as u32;
                                GAME.window_y = y as u32;
                            }
                            _ => {}
                        }
                    }
                    event::Event::KeyDown { timestamp, window_id, keycode, scancode, keymod, repeat } => {
                        if let Some(key) = keycode {
                            
                            match key {
                                keyboard::Keycode::Backquote => Game::toggle_cursor_visibility(),
                                _ => {}
                            }
                        }
                    }
                    event::Event::KeyUp { timestamp, window_id, keycode, scancode, keymod, repeat } => {
                        if let Some(key) = keycode {
                            
                        }
                    }
                    event::Event::MouseButtonDown { timestamp, window_id, which, mouse_btn, clicks, x, y } => {
                        
                    }
                    event::Event::MouseButtonUp { timestamp, window_id, which, mouse_btn, clicks, x, y } => {

                    }
                    event::Event::JoyButtonDown { timestamp, which, button_idx } => {

                    }
                    event::Event::JoyButtonUp { timestamp, which, button_idx } => {

                    }
                    event::Event::JoyAxisMotion { timestamp, which, axis_idx, value } => {

                    }
                    event::Event::JoyHatMotion { timestamp, which, hat_idx, state } => {

                    }
                    event::Event::JoyDeviceAdded { timestamp, which } => {

                    }
                    event::Event::MouseMotion { timestamp, window_id, which, mousestate, x, y, xrel, yrel } => {
                        if !Game::cursor_is_hidden()
                        {
                            let p_input = Game::get_input_sys();
                            let mut input = p_input.lock();
                            input.cursor_x.push(x as f32);
                            input.cursor_y.push(y as f32);
                            input.cursor_x.update();
                            input.cursor_y.update();
                            println!("x: {}", input.cursor_x.get_position());
                            println!("y: {}", input.cursor_y.get_position());
                            let mut new_x = x;
                            let mut new_y = y;
                            if x <= 1 {
                                new_x = GAME.window_x as i32 / 2;
                                input.cursor_x.push(new_x as f32);
                                RenderPipelineSystem::set_mouse_position(new_x, new_y);
                                
                            }
                            if y <= 1 {
                                new_y = GAME.window_y as i32 / 2;
                                input.cursor_y.push(new_y as f32);
                                RenderPipelineSystem::set_mouse_position(new_x, new_y);
                                
                            }
                            if x >= GAME.window_x as i32 - 1{
                                new_x = GAME.window_x as i32 / 2;
                                RenderPipelineSystem::set_mouse_position(new_x, new_y);
                                input.cursor_x.push(new_x as f32);
                            }
                            if y >= GAME.window_y as i32  - 1{
                                new_y = GAME.window_y as i32 / 2;
                                RenderPipelineSystem::set_mouse_position(new_x, new_y);
                                input.cursor_y.push(new_y as f32);
                            }

                            
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
        println!("{}", "Starting Event Thread!!".yellow());
        this.ready = true;
    }

    pub fn send_events(&mut self, events: &mut Vec<Arc<sdl2::event::Event>>)
    {
        self.event_pump.append(events);
    }

}