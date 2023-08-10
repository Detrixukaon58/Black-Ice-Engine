use std::{any::TypeId, future};
use std::collections::HashMap;
use std::any::Any;
use std::string;
use crate::common::angles::QuatConstructor;
use crate::common::vertex::V3New;
use crate::common::{*, mesh::Mesh, components::{component_system::ComponentSystem, entity::entity_system::*}};
use std::sync::{Arc, Mutex};
use ash::extensions::ext::DebugUtils;
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
    fn addClass(&mut self, class: Box<Register<>>){
        

        self.reg.insert(Box::new((*class.name)), class);
    }
}

#[derive(Clone)]
pub struct Register<>{
    pub RFID: Box<&'static str>,
    pub name : Box<&'static str>,
    pub desc : Box<&'static str>,
    props: HashMap<String, Box<Property<>>>,
    pointers: HashMap<String, Box<Pointer<>>>,
    funcs: HashMap<String, Box<Function<>>>,
    pub typeId: TypeId,
    pub reference: Box<&'static dyn Base>
}

#[derive(Clone)]
pub struct Property<>{
    pub name: Box<&'static str>,
    pub desc: Box<&'static str>,
    pub reference: Box<&'static dyn Base>,
    pub refType: TypeId
}

#[derive(Clone)]
pub struct Function<> {
    pub name: Box<&'static str>,
    pub desc: Box<&'static str>,
    pub paramTypes: Vec<TypeId>,
    pub reference: Box<&'static dyn Base>,
    pub outputType: TypeId

}

/// Anything provided as a heap loaction by Game must be stored in a Pointer type reference. This is so that the game can access it's registration for saving. E.g any component referenced in another must be placed in a Pointer type in order to be saved and reflected
#[derive(Clone)]
pub struct Pointer<> {
    pub name: Box<&'static str>,
    pub desc: Box<&'static str>,
    pub reference: Ptr<Register>,
    pub refType: TypeId
}

pub trait Registration<> {

    fn new<T: Base>(reference: Box<&'static T>) -> Register<>;

    fn register<T: Base>(&self, ourReg: &dyn Fn() -> Box<Register<>>);

    fn getProp(&self, name: &str) -> Box<&dyn Any>;
    fn getFunc(&self, name: &str) -> Box<&dyn Any>;
    fn addProp(&mut self, property: Property<>);
    fn addPointer(&mut self, pointer: Pointer<>);
    fn addFunc(&mut self, function: Function<>);

}

impl Registration<> for Register<> {

    fn new<T: Base>(reference: Box<&'static T>) -> Register<> {
        return Register { RFID: Box::new(""),
            name: Box::new(""), desc: Box::new(""),
            props: HashMap::<String, Box<Property<>>>::new(),
            pointers: HashMap::<String, Box<Pointer<>>>::new(),
            funcs: HashMap::<String, Box<Function<>>>::new(),
            typeId: TypeId::of::<T>(),
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
    fn registerReflect(&'static self) -> Ptr<Register<>>;
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

// This is always static(mustn't be created non-statically)
pub struct Game {

    pub gameName: Arc<Mutex<String>>,
    pub REGISTRAR: components::component_system::ComponentRef<Registry>,
    RENDER_SYS: components::component_system::ComponentRef<RenderPipelineSystem>,
    ENTITY_SYS: components::component_system::ComponentRef<EntitySystem>,
    pub STATUS: Arc<Mutex<StatusCode>>,
    pub sdl: sdl2::Sdl,
    pub window: sdl2::video::Window,
    pub video: sdl2::VideoSubsystem,
}

impl Game {
    pub unsafe fn isExit() -> bool {
        *GAME.STATUS.lock().unwrap() == StatusCode::CLOSE
    }

    pub fn new() -> Game{
        let reg = components::component_system::ComponentRef_new(Registry {reg: Lazy::new(
            || {HashMap::<Box<&str>,Box<Register>>::new()}
        )});
        let renderSys = components::component_system::ComponentRef_new(RenderPipelineSystem::new());

        let sdl = init().expect("Failed to initialise SDL!!");
        let video = sdl.video().expect("Failed to get video.");
        let window = video.window("Game Window", 800, 600)
            .position_centered()
            .vulkan()
            .resizable()
            .build()
            .expect("Failed to build window!")
        ;

        let ent_sys = components::component_system::ComponentRef_new(EntitySystem::new());

        Game { 
            gameName: Arc::new(Mutex::new(String::from("Game Name"))), 
            REGISTRAR: reg, 
            RENDER_SYS: renderSys, 
            ENTITY_SYS: ent_sys,
            STATUS: Arc::new(Mutex::new(StatusCode::INITIALIZE)),
            sdl: sdl,
            window: window,
            video: video,
        }
    }

    pub fn init(&'static mut self) {
        // Set up thread pool

        let runner = async{

            let mut event_pump = self.sdl.event_pump().expect("Failed to load event pump!");
            let p_render_sys = self.RENDER_SYS.clone();
            let p_entity_sys = self.ENTITY_SYS.clone();
            let renderJoinHandle = std::thread::spawn(|| {RenderPipelineSystem::init(p_render_sys)});
            let entity_Join_Handle = std::thread::spawn(|| {EntitySystem::init(p_entity_sys)});
            let p_ent_sys_2 = self.ENTITY_SYS.clone();
            let mut ent_sys_2 = p_ent_sys_2.lock().unwrap();
            let mut entity_params = components::entity::entity_system::EntityParams {
                position: vertex::Vec3::new(0, 0, 0),
                rotation: angles::Quat::new(0.0, 0.0, 1.0, 0.0),
                scale: vertex::Vec3::new(0, 0, 0),

            };
            let p_entity = ent_sys_2.add_entity(entity_params);
            drop(ent_sys_2);
            let def: serde_json::Value = serde_json::from_str(r#"
            {
                "image_file": {
                    "path" : "ASSET:images\nemissa_hitomi.png"
                }
            }
            "#).unwrap();
            println!("{}", def["image_file"]);
            Entity::add_component::<components::entity::image_component::Image>(p_entity, def);
            // here we loop for the events
            'running: loop {
                for event in event_pump.poll_iter() {
                    match event {
                        event::Event::Quit {..} =>  {
                            unsafe{Game::set_status(StatusCode::CLOSE);}
                            println!("Close sent");
                            break 'running;
                        }
                        _ => continue
                    }
                }
            }

            renderJoinHandle.join();
            entity_Join_Handle.join();
            println!("Exiting Game!!");
        };

        futures::executor::block_on(runner);
        
    }

    unsafe fn set_status(status: StatusCode){
        *GAME.STATUS.lock().unwrap() = status.clone();
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

    pub unsafe fn get_render_sys() -> components::component_system::ComponentRef<RenderPipelineSystem> {
        GAME.RENDER_SYS.clone()
    }

    pub unsafe fn get_entity_sys() -> components::component_system::ComponentRef<EntitySystem> {
        GAME.ENTITY_SYS.clone()
    }

}

pub static mut GAME: Lazy<Game> = Lazy::new( || {Game::new()});