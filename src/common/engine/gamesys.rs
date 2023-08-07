use std::any::TypeId;
use std::collections::HashMap;
use std::any::Any;
use std::string;
use crate::common::{*, mesh::Mesh};
use std::sync::{Arc, Mutex};
use ash::extensions::ext::DebugUtils;
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

#[derive(PartialEq)]
pub enum SatusCode{
    RUNNING,
    CLOSE,
    INITIALIZE,
}

// This is always static(mustn't be created non-statically)
pub struct Game {

    pub gameName: Arc<Mutex<String>>,
    pub REGISTRAR: Box<Registry>,
    pub RENDER_SYS: Box<RenderPipelineSystem>,
    pub STATUS: SatusCode,
    pub sdl: sdl2::Sdl,
    pub window: sdl2::video::Window,
    pub video: sdl2::VideoSubsystem,
}

impl Game {
    pub fn isExit(&self) -> bool {
        self.STATUS == SatusCode::CLOSE
    }

    pub fn new() -> Game{
        let reg = Box::new(Registry {reg: Lazy::new(
            || {HashMap::<Box<&str>,Box<Register>>::new()}
        )});
        let renderSys = Box::new(RenderPipelineSystem::new());

        let sdl = init().expect("Failed to initialise SDL!!");
        let video = sdl.video().expect("Failed to get video.");
        let window = video.window("Game Window", 800, 600)
            .position_centered()
            .vulkan()
            .resizable()
            .build()
            .expect("Failed to build window!")
        ;

        Game { 
            gameName: Arc::new(Mutex::new(String::from("Game Name"))), 
            REGISTRAR: reg, 
            RENDER_SYS: renderSys, 
            STATUS: SatusCode::INITIALIZE,
            sdl: sdl,
            window: window,
            video: video,
        }
    }

    pub fn init(&'static mut self) {
        // create a window
        let mut event_pump = self.sdl.event_pump().expect("Failed to load event pump!");
        let renderJoinHandle = self.RENDER_SYS.init();
        
        self.STATUS = SatusCode::RUNNING;
        // here we loop for the events
        'running: loop {
            for event in event_pump.poll_iter() {
                match event {
                    event::Event::Quit {..} =>  break 'running,
                    _ => continue
                }
            }

            
        }
        self.STATUS = SatusCode::CLOSE;
        renderJoinHandle.join();
        
    }


}

pub static mut GAME: Lazy<Game> = Lazy::new( || {Game::new()});