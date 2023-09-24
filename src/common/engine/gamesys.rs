#![allow(unused)]
#![allow(non_snake_case)]
use std::{any::TypeId, future};
use std::collections::HashMap;
use std::any::Any;
use std::string;
use crate::common;
use crate::common::angles::*;
use crate::common::components::component_system::Constructor;
use crate::common::engine::event_system::EventSystem;
use crate::common::matrices::{M34, QuatToMat33};
use crate::common::vertex::{V3New, V3Meth};
use crate::common::{*, mesh::Mesh, components::{component_system::*, entity::{entity_system::*, *}}};
use std::sync::Arc;
use colored::Colorize;
use parking_lot::*;
use futures::join;
use sdl2::*;

use once_cell::sync::Lazy;
use crate::common::engine::{pipeline::*, input, event_system};
#[cfg(feature = "vulkan")] use ash::*;

use super::input::InputSystem;
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

#[derive(Clone)]
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

    pub fn get_position(&self) -> f32 
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
    INPUT_SYS: Arc<Mutex<InputSystem>>,
    EVENT_SYS: Arc<Mutex<EventSystem>>,
    pub STATUS: Arc<Mutex<StatusCode>>,
    pub sdl: sdl2::Sdl,
    pub mouse: sdl2::mouse::MouseUtil,
    pub keyboard: sdl2::keyboard::KeyboardUtil,
    pub window_x: u32,
    pub window_y: u32,
    show_cursor: bool,
}

impl Game {
    pub unsafe fn isExit() -> bool {
        'run: loop {
            let status = match GAME.STATUS.try_lock() {
                Some(v) => v,
                None => continue 'run
            };
            return *status == StatusCode::CLOSE;
        }
    }

    pub fn new() -> Game{
        let reg = components::component_system::ComponentRef_new(Registry {reg: Lazy::new(
            || {HashMap::<Box<&str>,Box<Register>>::new()}
        )});
        

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
        let input_sys = Arc::new(Mutex::new(InputSystem::new(x / 2, y / 2)));
        let event_system = Arc::new(Mutex::new(EventSystem::new()));
        let render_sys = Arc::new(RwLock::new(RenderPipelineSystem::new(&sdl, video, window)));
        Game { 
            gameName: Arc::new(Mutex::new(String::from("Game Name"))), 
            REGISTRAR: reg, 
            RENDER_SYS: render_sys, 
            ENTITY_SYS: ent_sys,
            INPUT_SYS: input_sys,
            EVENT_SYS: event_system,
            STATUS: Arc::new(Mutex::new(StatusCode::INITIALIZE)),
            sdl: sdl,
            mouse: mouse,
            keyboard: keyboard,
            window_x: x,
            window_y: y,
            show_cursor: false,
        }
    }

    pub fn init(&'static mut self) {
        // Set up thread pool

        let runner = async{

            
            let p_render_sys = self.RENDER_SYS.clone();
            let p_entity_sys = self.ENTITY_SYS.clone();
            let p_input_sys = self.INPUT_SYS.clone();
            let p_event_system = self.EVENT_SYS.clone();
            let event_join_handdle = std::thread::Builder::new().name("Event".to_string()).spawn(|| {EventSystem::init(p_event_system)}).unwrap();
            let render_join_handle = std::thread::Builder::new().name(String::from("render")).spawn(|| {RenderPipelineSystem::init(p_render_sys)}).expect("Failed to create render thread!!");
            let entity_join_handle = std::thread::Builder::new().name(String::from("entity")).spawn(|| {EntitySystem::init(p_entity_sys)}).expect("Failed to start entity thread!!");
            let input_join_handle = std::thread::Builder::new().name("Input".to_string()).spawn(|| {InputSystem::init(p_input_sys)}).unwrap();
            let p_ent_sys_2 = self.ENTITY_SYS.clone();
            let p_rend_sys_2 = self.RENDER_SYS.clone();
            let p_input_sys_2 = self.INPUT_SYS.clone();
            let p_event_sys_2 = self.EVENT_SYS.clone();
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
            for i in 0..10 {
                let mut p_mesh = p_entity2.add_component::<mesh_component::MeshComponent>(mesh_def.clone());
                let mut mesh = p_mesh.lock();
                // mesh.triangles();
                mesh.square();
                mesh.translate(Vec3::new(15.0 * i as f32, 0.0, 0.0));
                drop(mesh);
                v_p_mesh.push(p_mesh);
            }
            let mut cam = p_cam.lock();
            let mut input = self.INPUT_SYS.lock();

            cam.set_position(Vec3::new(0.0, 0.0, 1.0));
            cam.set_rotation(Quat::euler(Ang3::new(
                input.cursor_x.get_position() / 25.0, 
                input.cursor_y.get_position() * input.cursor_x.get_position().cos() / 25.0,
                input.cursor_y.get_position() * input.cursor_x.get_position().sin() / 25.0,
            )));
            drop(input);
            // cam.set_rotation(Quat::euler(Ang3::new(0.0, 90.0, 0.0)));
            drop(cam);
            EntitySystem::start(p_ent_sys_2.clone());
            RenderPipelineSystem::start(p_rend_sys_2.clone());
            InputSystem::start(p_input_sys_2.clone());
            EventSystem::start(p_event_sys_2.clone());

            // here we loop for the events
            // let mut forward = Vec3::new(1.0, 0.0, 0.0);
            loop{
                if unsafe{Game::isExit()} {
                    break;
                }
                let mut pp_event_pump = self.sdl.event_pump();
                let mut p_event_pump = pp_event_pump.unwrap();
                let mut event_pump = p_event_pump.poll_iter();
                let mut events = event_pump.map(|f| Arc::new(f)).collect::<Vec<Arc<sdl2::event::Event>>>();
                let mut event_sys = p_event_sys_2.lock();
                event_sys.send_events(&mut events);
            }

            

            render_join_handle.join();
            entity_join_handle.join();
            input_join_handle.join();
            println!("Exiting Game!!");
        };

        futures::executor::block_on(runner);
        
    }

    pub unsafe fn set_status(status: StatusCode){
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

    pub unsafe fn cursor_is_hidden() -> bool 
    {
        GAME.show_cursor
    }

    pub unsafe fn get_render_sys() -> Arc<RwLock<RenderPipelineSystem>> {
        GAME.RENDER_SYS.clone()
    }

    pub unsafe fn get_entity_sys() -> components::component_system::ComponentRef<EntitySystem> {
        GAME.ENTITY_SYS.clone()
    }

    pub unsafe fn get_input_sys() -> Arc<Mutex<InputSystem>> {
        GAME.INPUT_SYS.clone()
    }

    pub fn toggle_cursor_visibility()
    {   
        unsafe{
            let show_cursor = !GAME.show_cursor;
            println!("show_cursor: {}", show_cursor);
            GAME.mouse.show_cursor(show_cursor);
            GAME.mouse.capture(!show_cursor);
            GAME.show_cursor = show_cursor;
            let p_rend_sys = Game::get_render_sys();
            let mut rend_sys = p_rend_sys.read();
            let p_window = rend_sys.window.clone();
            let mut window = p_window.lock();
            drop(rend_sys);
            window.hide();
            std::thread::sleep(std::time::Duration::from_millis(1));
            window.show();
        }
    }

}

pub static mut GAME: Lazy<Game> = Lazy::new( || {Game::new()});