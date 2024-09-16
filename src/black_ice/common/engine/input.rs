#![allow(unused)]


use std::{sync::Arc, thread::JoinHandle};
use colored::Colorize;
use parking_lot::*;
use sdl2::{*, sys::*, mouse::MouseButton};

use crate::black_ice::common::{engine::gamesys::*, components::entity::entity_system::*};

#[derive(Clone)]
pub enum MouseCode {
    Mb1,
    Mb2,
    Mb3,
    Mb4,
    Middle,

}

#[derive(Clone)]
pub enum JoyButton {
    PsCross,
    PsCircle,
    PsSquare,
    PsTriangle,
    L1,
    L2,
    L3,
    R1,
    R2,
    R3,
    XboxA,
    XboxB,
    XboxX,
    XboxY,
    SwitchA,
    SwitchB,
    SwitchX,
    SwitchY,
    A,
    B,
    X,
    Y
}

type Axis = u8;
type ControllerId = u32;

#[derive(Clone)]
pub enum InputValue {
    KeyDown(KeyCode, u32),
    KeyUp(KeyCode, u32),
    MouseDown(MouseCode, u32),
    MouseUp(MouseCode, u32),
    JoyDown(ControllerId, JoyButton, u32),
    JoyUp(ControllerId, JoyButton, u32),
    JoyAxis(ControllerId, Axis, u32),

}



pub struct InputMap<T, U>{
    inner: Vec<(T, U)>,
}

pub struct Input {

    button_maps: InputMap<InputValue, *mut dyn Fn(InputValue) -> ()>

}

unsafe impl Send for Input {}
unsafe impl Sync for Input {}

pub struct InputSystem {
    input_buffer: Vec<InputValue>,
    input_event_handlers: Vec<Arc<Mutex<Input>>>,
    pub cursor_x: Avg<f32>,
    pub cursor_y: Avg<f32>,
    ready: bool,

}

impl InputSystem {

    pub fn init(this: Arc<Mutex<Self>>) -> i32 {

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
            
            let this = p_this.lock();
            let input_buffer = this.input_buffer.clone();
            let input_event_handlers = this.input_event_handlers.clone();
            drop(this);
            for inp in &input_buffer {
                for p_input_handlers in &input_event_handlers {
                    let input_handlers = p_input_handlers.lock();

                }
            }

        
        }

        0
    }

    pub fn new(x: u32, y: u32) -> Self {
        unsafe{
            let mut cursor_x = Avg::<f32>::new();
            let mut cursor_y = Avg::<f32>::new();
            cursor_x.push(x as f32 / 2.0);
            cursor_y.push(y as f32 / 2.0);

            Self { 
                input_buffer: Vec::new(), 
                input_event_handlers: Vec::new(), 
                cursor_x: cursor_x,
                cursor_y: cursor_y,
                ready: true,
            }
        }
    }

    pub fn get_cursor() -> (Avg<f32>, Avg<f32>) {
        unsafe
        {
            let p_input = Env::get_input_sys();
            let mut input = p_input.lock();
            (input.cursor_x.clone(), input.cursor_y.clone())
        }
    }

    pub fn reset_cursor() {
        unsafe
        {
            let p_input = Env::get_input_sys();
            let mut input = p_input.lock();
            input.cursor_x.reset();
            input.cursor_y.reset();
        }
    }

    pub fn start(p_this: Arc<Mutex<Self>>) {
        let mut this = p_this.lock();
        println!("{}", "Starting Input Thread!!".yellow());
        this.ready = true;
    }

}