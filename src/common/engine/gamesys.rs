#![allow(unused)]
#![allow(non_snake_case)]
use std::{any::TypeId, future};
use std::collections::HashMap;
use std::any::Any;
use std::string;
use crate::common;
use crate::common::angles::*;
use crate::common::components::component_system::Constructor;
use crate::common::matrices::{M34, QuatToMat33};
use crate::common::vertex::{V3New, V3Meth};
use crate::common::{*, mesh::Mesh, components::{component_system::*, entity::{entity_system::*, *}}};
use std::sync::Arc;
use colored::Colorize;
use parking_lot::*;
use futures::join;
use sdl2::*;

use once_cell::sync::Lazy;
use crate::common::engine::pipeline::*;
#[cfg(feature = "vulkan")] use ash::*;
pub trait BaseToAny: 'static {
    fn as_any(&self) -> &dyn Any;
}

impl<T: 'static> BaseToAny for T {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub trait AnyToBase: 'static {
    fn as_base(&self) -> &dyn Base;
}

pub trait Base: BaseToAny + Sync{
}

//static mut REGISTRAR: Lazy<Box<Registry>> = Lazy::new(|| Box::<Registry>::new(Registry { reg: Lazy::new(||HashMap::<Box::<&str>, Box::<Register<>>>::new())}));

pub struct Registry {
    // (str: id, Box<Register<>>: Register Def)
    pub reg: Lazy<HashMap<Box<&'static str>, Box<Register<>>>>,

}

impl Registry<> {
    fn add_class(&mut self, class: Box<Register<>>){
        

        self.reg.insert(Box::new((*class.name)), class);
    }
}

#[derive(Clone)]
pub struct Register<>{
    pub rfid: Box<&'static str>,
    pub name : Box<&'static str>,
    pub desc : Box<&'static str>,
    props: HashMap<String, Box<Property<>>>,
    pointers: HashMap<String, Box<Pointer<>>>,
    funcs: HashMap<String, Box<Function<>>>,
    pub type_id: TypeId,
    pub reference: Box<&'static dyn Base>
}

#[derive(Clone)]
pub struct Property<>{
    pub name: Box<&'static str>,
    pub desc: Box<&'static str>,
    pub reference: Box<&'static dyn Base>,
    pub ref_type: TypeId
}

#[derive(Clone)]
pub struct Function<> {
    pub name: Box<&'static str>,
    pub desc: Box<&'static str>,
    pub param_types: Vec<TypeId>,
    pub reference: Box<&'static dyn Base>,
    pub output_type: TypeId

}

/// Anything provided as a heap loaction by Game must be stored in a Pointer type reference. This is so that the game can access it's registration for saving. E.g any component referenced in another must be placed in a Pointer type in order to be saved and reflected
#[derive(Clone)]
pub struct Pointer<> {
    pub name: Box<&'static str>,
    pub desc: Box<&'static str>,
    pub reference: Ptr<Register>,
    pub ref_type: TypeId
}

pub trait Registration<> {

    fn new<T: Base>(reference: Box<&'static T>) -> Register<>;

    fn register<T: Base>(&self, our_reg: &dyn Fn() -> Box<Register<>>);

    fn getProp(&self, name: &str) -> Box<&dyn Any>;
    fn getFunc(&self, name: &str) -> Box<&dyn Any>;
    fn addProp(&mut self, property: Property<>);
    fn addPointer(&mut self, pointer: Pointer<>);
    fn addFunc(&mut self, function: Function<>);

}

impl Registration<> for Register<> {

    fn new<T: Base>(reference: Box<&'static T>) -> Register<> {
        return Register { rfid: Box::new(""),
            name: Box::new(""), desc: Box::new(""),
            props: HashMap::<String, Box<Property<>>>::new(),
            pointers: HashMap::<String, Box<Pointer<>>>::new(),
            funcs: HashMap::<String, Box<Function<>>>::new(),
            type_id: TypeId::of::<T>(),
            reference: Box::new(*reference)};
    }

    fn register<T: Base>(&self, ourReg: &dyn Fn() -> Box<Register<>>){
        let mut class = ourReg();
        unsafe{
            //REGISTRAR.addClass(class);
        }
        
    }

    fn getProp(&self, name: &str) -> Box<&dyn Any>{
        let register = self;

        return Box::new(register.props.get(&name.to_string()).unwrap());

    }
    fn getFunc(&self, name: &str) -> Box<&dyn Any>{

        let register = self;

        return Box::new(register.funcs.get(&name.to_string()).unwrap());

    }

    fn addProp(&mut self, property: Property<>){
        let mut register = self;

        register.props.insert(property.name.to_string(), Box::new(property));
        

    }
    fn addPointer(&mut self, pointer: Pointer<>) {
        let mut register = self;
        register.pointers.insert(pointer.name.to_string(), Box::new(pointer));
    }
    fn addFunc(&mut self, function: Function<>){
        let mut register = self;

        register.funcs.insert(function.name.to_string(), Box::new(function));
    }
}

#[derive(Clone)]
pub struct Ptr<T> {
    pub b: Box<T>,
}

pub trait Reflection: Base {
    fn register_reflect(&'static self) -> Ptr<Register<>>;
}


// Initialisers

/// This trait is used to ensure that All 
trait Initialiser {
    fn init(&mut self);
}

#[derive(PartialEq, Clone)]
pub enum StatusCode{
    RUNNING,
    CLOSE,
    INITIALIZE,
}

pub struct Avg<T> {
    inner: Vec<T>,
    init: f32
}

impl Avg<f32> {
    pub fn push(&mut self, value: f32)
    {
        if self.inner.len() > 30 {
            self.inner.remove(0);
        }
        self.inner.push(value);
    }

    pub fn average(&self) -> f32 {
        self.inner.iter().sum::<f32>() / self.inner.len() as f32
    }

    pub fn update(&mut self)
    {
        let change = self.change();
        self.init += change;
    }

    pub fn change(&mut self) -> f32 {
        if self.inner.len() > 1 {
            self.inner.reverse();
            let res = self.inner[0] - self.inner[1];
            // println!("{}, {}", self.inner[0], self.inner[1]);
            self.inner.reverse();
            
            res
            
        }
        else if !self.inner.is_empty(){
            self.inner[0]
        }
        else{
            0.0
        }
    }

    pub fn get_position(&mut self) -> f32 
    {
        self.init
    }

    pub fn new() -> Self {
        Self { inner: Vec::new(), init: 0.0}
    }
}

// This is always static(mustn't be created non-statically)
pub struct Game {

    pub gameName: Arc<Mutex<String>>,
    pub REGISTRAR: components::component_system::ComponentRef<Registry>,
    RENDER_SYS: Arc<RwLock<RenderPipelineSystem>>,
    ENTITY_SYS: components::component_system::ComponentRef<EntitySystem>,
    pub STATUS: Arc<Mutex<StatusCode>>,
    pub sdl: sdl2::Sdl,
    pub window: sdl2::video::Window,
    pub video: sdl2::VideoSubsystem,
    pub mouse: sdl2::mouse::MouseUtil,
    pub keyboard: sdl2::keyboard::KeyboardUtil,
    pub event_pump: Option<sdl2::EventPump>,
    pub window_x: u32,
    pub window_y: u32,
    show_cursor: bool,
    pub cursor_x: Avg<f32>,
    pub cursor_y: Avg<f32>
}

impl Game {
    pub unsafe fn isExit() -> bool {
        *GAME.STATUS.lock() == StatusCode::CLOSE
    }

    pub fn new() -> Game{
        let reg = components::component_system::ComponentRef_new(Registry {reg: Lazy::new(
            || {HashMap::<Box<&str>,Box<Register>>::new()}
        )});
        let render_sys = Arc::new(RwLock::new(RenderPipelineSystem::new()));

        let sdl = init().expect("Failed to initialise SDL!!");
        let video = sdl.video().expect("Failed to get video.");
        let x = 800;
        let y = 600;
        #[cfg(feature = "vulkan")]
        let window = video.window("Game Window", x, y)
            .position_centered()
            .vulkan()
            .resizable()
            .build()
            .expect("Failed to build window!")
        ;

        #[cfg(feature = "opengl")]
        let window = video.window("Game Window", x, y)
            .position_centered()
            .opengl()
            .resizable()
            .build()
            .expect("Failed to build window!")
        ;
        let ent_sys = components::component_system::ComponentRef_new(EntitySystem::new());
        let mouse = sdl.mouse();
        let keyboard = sdl.keyboard();
        mouse.show_cursor(false);
        mouse.capture(true);
        mouse.warp_mouse_in_window(&window, x as i32 / 2, y as i32 / 2);
        let mut cursor_x = Avg::<f32>::new();
        let mut cursor_y = Avg::<f32>::new();
        cursor_x.push(x as f32 / 2.0);
        cursor_y.push(y as f32 / 2.0);
        Game { 
            gameName: Arc::new(Mutex::new(String::from("Game Name"))), 
            REGISTRAR: reg, 
            RENDER_SYS: render_sys, 
            ENTITY_SYS: ent_sys,
            STATUS: Arc::new(Mutex::new(StatusCode::INITIALIZE)),
            sdl: sdl,
            window: window,
            video: video,
            mouse: mouse,
            keyboard: keyboard,
            event_pump: None,
            window_x: x,
            window_y: y,
            show_cursor: false,
            cursor_x: cursor_x,
            cursor_y: cursor_y,
        }
    }

    pub fn init(&'static mut self) {
        // Set up thread pool

        let runner = async{

            self.event_pump = Some(self.sdl.event_pump().expect("Failed to load event pump!"));
            let p_render_sys = self.RENDER_SYS.clone();
            let p_entity_sys = self.ENTITY_SYS.clone();
            let render_join_handle = std::thread::Builder::new().name(String::from("render")).spawn(|| {RenderPipelineSystem::init(p_render_sys)}).expect("Failed to create render thread!!");
            let entity_join_handle = std::thread::Builder::new().name(String::from("entity")).spawn(|| {EntitySystem::init(p_entity_sys)}).expect("Failed to start entity thread!!");
            let p_ent_sys_2 = self.ENTITY_SYS.clone();
            let p_rend_sys_2 = self.RENDER_SYS.clone();
            let mut ent_sys_2 = p_ent_sys_2.lock();
            let mut entity_params = components::entity::entity_system::EntityParams {
                position: vertex::Vec3::new(0, 0, 0),
                rotation: angles::Quat::axis_angle(Vec3::new(0.0, 0.0, 0.0), 0.0),
                scale: vertex::Vec3::new(1.0, 1.0, 1.0),

            };
            let mut p_entity = ent_sys_2.add_entity(entity_params);
            let mut entity_params2 = components::entity::entity_system::EntityParams {
                position: vertex::Vec3::new(0, 0, 0),
                rotation: angles::Quat::axis_angle(Vec3::new(0.0, 0.0, 0.0), 0.0),
                scale: vertex::Vec3::new(1.0, 1.0, 1.0),

            };
            let mut p_entity2 = ent_sys_2.add_entity(entity_params2);
            drop(ent_sys_2);
            let def: common::components::component_system::Value = common::components::component_system::ValueBuilder::new().from_str(r#"
            {
                "image_file": {
                    "path" : "ASSET:\\images\\nemissa_hitomi.png"
                }
            }
            "#).build();
            println!("{}", def["image_file"]);
            p_entity.add_component::<components::entity::image_component::Image>(Arc::new(def));
            let cam_def = components::entity::camera_component::CameraComponent::default_constuctor_definition();
            

            let pipe = PipelineParams {name: "Test Pipeline".to_string(), layer: 0};

            use mesh::*;
            let mesh_def = mesh_component::MeshComponent::default_constuctor_definition();
            unsafe {
                let pipe_id = RenderPipelineSystem::resgister_pipeline(pipe);
            }
            let p_cam = p_entity.add_component::<components::entity::camera_component::CameraComponent>(cam_def);
            let mut v_p_mesh = Vec::<ComponentRef<mesh_component::MeshComponent>>::new();
            for i in 0..5 {
                let mut p_mesh = p_entity2.add_component::<mesh_component::MeshComponent>(mesh_def.clone());
                let mut mesh = p_mesh.lock();
                // mesh.triangles();
                mesh.square();
                mesh.translate(Vec3::new(1.0 * i as f32, 0.0, 0.0));
                drop(mesh);
                v_p_mesh.push(p_mesh);
            }
            let mut cam = p_cam.lock();

            cam.set_position(Vec3::new(0.0, 0.0, 10.0));
            // cam.set_rotation(Quat::euler(Ang3::new(0.0, 0.0, 0.0)));
            drop(cam);
            EntitySystem::start(p_ent_sys_2.clone());
            RenderPipelineSystem::start(p_rend_sys_2.clone());

            // here we loop for the events
            // let mut forward = Vec3::new(1.0, 0.0, 0.0);
            'running: loop {
                for event in self.event_pump.as_mut().unwrap().poll_iter() {
                    match event {
                        event::Event::Quit {..} =>  {
                            unsafe{Game::set_status(StatusCode::CLOSE);}
                            println!("Close sent");
                            break 'running;
                        }
                        event::Event::Window { timestamp, window_id, win_event } => {
                            match win_event {
                                event::WindowEvent::Resized(x, y) => {
                                    self.window_x = x as u32;
                                    self.window_y = y as u32;
                                }
                                _ => {}
                            }
                        }
                        event::Event::KeyDown { timestamp, window_id, keycode, scancode, keymod, repeat } => {
                            if let Some(key) = keycode {
                                use keyboard::Keycode::*;
                                match key {
                                    W => {
                                        // let mut cam = p_cam.lock();

                                        // let ang = Ang3::new(0.0, 10.0, 0.0);
                                        // cam.rotate(Quat::euler(ang));
                                        p_entity.translate(Vec3::new(10.0, 0.0, 0.0));

                                        println!("{}", "Move Forward".green());

                                    }
                                    S => {
                                        // let mut cam = p_cam.lock();

                                        // let ang = Ang3::new(0.0, -10.0, 0.0);
                                        // cam.rotate(Quat::euler(ang));
                                        p_entity.translate(Vec3::new(-10.0, 0.0, 0.0));

                                        println!("{}", "Move Backward".green());

                                    }
                                    A => {
                                        // let mut cam = p_cam.lock();

                                        // let ang = Ang3::new(10.0, 0.0, 0.0);
                                        // cam.rotate(Quat::euler(ang));
                                        p_entity.translate(Vec3::new(0.0, -10.0, 0.0));
                                        println!("{}", "Move Left".green());

                                    }
                                    D => {
                                        // let mut cam = p_cam.lock();

                                        // let ang = Ang3::new(-10.0, 0.0, 0.0);
                                        // cam.rotate(Quat::euler(ang));
                                        p_entity.translate(Vec3::new(0.0, 10.0, 0.0));
                                        println!("{}", "Move Right".green());

                                    }
                                    Backquote => {
                                        self.show_cursor = ! self.show_cursor;
                                        self.mouse.capture(self.show_cursor);
                                        self.mouse.show_cursor(self.show_cursor);
                                        self.window.hide();
                                        self.window.show();
                                    }
                                    Q => {
                                        p_entity.translate(Vec3::new(0.0, 0.0, 10.0));
                                    }
                                    E => {
                                        p_entity.translate(Vec3::new(0.0, 0.0, -10.0));
                                    }
                                    _ => {}
                                }
                            }
                        }
                        event::Event::MouseMotion { timestamp, window_id, which, mousestate, x, y, xrel, yrel } => {
                            if !self.show_cursor{
                                let mut cam = p_cam.lock();
                                self.cursor_x.push(x as f32);
                                self.cursor_y.push(y as f32);
                                self.cursor_x.update();
                                self.cursor_y.update();
                                let ang = Ang3::new(-self.cursor_x.change(), 0.0, 0.0);
                                let quat = Quat::euler(ang);
                                let ang2 = quat.to_euler();
                                // println!("{}, {}, {}", ang2.y, ang2.p, ang2.r);
                                // println!("{}", quat.to_mat33().to_mat34());
                                println!("{}", self.cursor_x.change());
                                cam.rotate(quat);
                                // cam.set_rotation(quat);
                                // cam.rotate(Quat::euler(Ang3::new(0.0, 0.0, self.cursor_y.get_position())));
                                let mut new_x = x;
                                let mut new_y = y;
                                if x <= 1 {
                                    new_x = self.window_x as i32 - 4;
                                    self.mouse.warp_mouse_in_window(&self.window, new_x, new_y);
                                    self.cursor_x.push(-new_x as f32);
                                }
                                if y <= 1 {
                                    new_y = self.window_y as i32 - 4;
                                    self.mouse.warp_mouse_in_window(&self.window, new_x, new_y);
                                    self.cursor_y.push(-new_y as f32);
                                }
                                if x >= self.window_x as i32 - 1{
                                    new_x = 4;
                                    self.mouse.warp_mouse_in_window(&self.window, new_x, new_y);
                                    self.cursor_x.push(-new_x as f32);
                                }
                                if y >= self.window_y as i32  - 1{
                                    new_y = 4;
                                    self.mouse.warp_mouse_in_window(&self.window, new_x, new_y);
                                    self.cursor_y.push(-new_y as f32);
                                }
                                
                                
                                drop(cam);
                            }
                        }
                        _ => continue
                    }
                }
                // let mut mesh = p_mesh.lock();
                // mesh.rotate(Quat::euler(1.0, 0.0, 0.0));
                // drop(mesh);
            }

            self.window.hide();

            render_join_handle.join();
            entity_join_handle.join();
            println!("Exiting Game!!");
        };

        futures::executor::block_on(runner);
        
    }

    unsafe fn set_status(status: StatusCode){
        *GAME.STATUS.lock() = status.clone();
        let p_rend = Game::get_render_sys().clone();
        let p_ent = Game::get_entity_sys().clone();

        RenderPipelineSystem::send_status(p_rend, status.clone());
        EntitySystem::send_status(p_ent, status.clone());

        //rend.send_status(status.clone());
        //drop(rend);
        //ent.send_status(status.clone());
        //drop(ent);
        return;
    }

    pub unsafe fn get_render_sys() -> Arc<RwLock<RenderPipelineSystem>> {
        GAME.RENDER_SYS.clone()
    }

    pub unsafe fn get_entity_sys() -> components::component_system::ComponentRef<EntitySystem> {
        GAME.ENTITY_SYS.clone()
    }

}

pub static mut GAME: Lazy<Game> = Lazy::new( || {Game::new()});