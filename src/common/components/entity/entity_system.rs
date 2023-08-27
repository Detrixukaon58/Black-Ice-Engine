// TODO: Make an entity registration system to allow for components to be registered to an entity
#![allow(unused)]
use std::{any::*, thread::JoinHandle, collections::*, sync::{*, atomic::AtomicU32}, future::*, pin::*, ops::DerefMut};
use serde::*;
use bitmask_enum::*;

use crate::common::{engine::{gamesys::*, threading::ThreadData}, vertex::*, angles::*, components::{component_system::*, entity}};

use colored::*;

use self::entity_event::*;

pub type EntityID = u32;

pub struct Entity {
    pub entity_id: EntityID,
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
    component_system: ComponentRef<Vec<ComponentRef<dyn BaseComponent>>>,
    thread_reciever: Arc<Mutex<Vec<EventThreadData>>>,
    is_dead: bool
}

unsafe impl Sync for Entity {}

impl Base for Entity{}

impl Default for Entity {
    fn default() -> Self {
        Self { 
            entity_id: Default::default(), 
            position: Default::default(), 
            rotation: Default::default(), 
            scale: Default::default(),
            component_system: ComponentRef_new(Vec::<ComponentRef<dyn BaseComponent>>::new()),
            thread_reciever: Arc::new(Mutex::new(Vec::new())),
            is_dead: false
        }
    }
}

impl Reflection for Entity{
    fn register_reflect(&'static self) -> Ptr<Register<>> {
        let mut registration:  Box<Register> = Box::new(Register::new(Box::new(self)));

        registration.addProp(Property { 
            name: Box::new("position"), 
            desc: Box::new("position of the Entity"), 
            reference: Box::new(&self.position), 
            ref_type: TypeId::of::<Vec3>() 
        });
        registration.addProp(Property { 
            name: Box::new("rotation"), 
            desc: Box::new("rotation of the Entity"), 
            reference: Box::new(&self.rotation), 
            ref_type: TypeId::of::<Quat>() 
        });
        registration.addProp(Property { 
            name: Box::new("scale"), 
            desc: Box::new("scale of the Entity"), 
            reference: Box::new(&self.scale), 
            ref_type: TypeId::of::<Vec3>() 
        });

        return Ptr {b: registration};
    }
}

impl Entity {
    pub fn add_component<T>(this: ComponentRef<Self>, definition: ConstructorDefinition) -> ComponentRef<T> where T: BaseComponent + Constructor<T> {
        unsafe{

            println!("{}", "Adding component to Entity!!".bright_red());
            let p_entity = this.clone();
            let component = T::construct(p_entity, &definition).unwrap();
            'test: loop{
                let entity = match this.try_lock() {
                    Ok(ent) => ent,
                    Err(err) => continue 'test
                };
                let p_comp_sys = entity.component_system.clone();
                let mut comp_sys = p_comp_sys.lock().unwrap();
                comp_sys.push(component.clone());
                drop(entity);
                drop(comp_sys);
                break;
            }
            println!("{}", "Added component to Entity!!".bright_red());
            component
        }
    }

    pub fn init<'a>(this: ComponentRef<Self>) -> i32 {
        Self::processing(this)
    }

    pub fn collect_component(p_this: ComponentRef<Vec<ComponentRef<dyn BaseComponent>>>, count: usize) -> ComponentRef<dyn BaseComponent> {
        let this = p_this.lock().unwrap();
        return this[count].clone();
    }

    fn len(p_this: ComponentRef<Vec<ComponentRef<dyn BaseComponent>>>) -> usize {
        let this = p_this.lock().unwrap();
        return this.len().clone();
    }

    fn processing(p_this: ComponentRef<Self>) -> i32 {
        'run: loop{
            let mut this = match p_this.try_lock() {
                Ok(ent) => ent,
                Err(err) => continue 'run
            };
            let p_recv = this.thread_reciever.clone();
            let mut recv = p_recv.try_lock();
            if let Ok(ref mut mutex) = recv {
                for th in mutex.as_slice() {
                    let data = th.clone();
                    
                    match data {
                        EventThreadData::Event(event) => {
                            let p_comp_sys = this.component_system.clone();
                            let mut i = 0;
                            while i < Self::len(p_comp_sys.clone()){
                                let p_component = Self::collect_component(p_comp_sys.clone(), i);
                                let mut component = p_component.lock().unwrap();
                                if(component.get_event_mask().contains(event.event_flag)){
                                    component.process_event(&event);
                                }
                                i += 1;
                            }
                            
                        },
                        EventThreadData::KillEvent() => {
                            let p_comp_sys = this.component_system.clone();
                            let comp_sys = p_comp_sys.lock().unwrap();
                            for p_component in  comp_sys.to_vec(){
                                drop(p_component);
                            }
                            this.is_dead = true;
                            break 'run;
                        }
                        _ => {},
                    }
                }
                mutex.clear();
            }
            drop(recv);
            drop(this);
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        println!("{}", "Killed Entity".red());
        std::thread::sleep(std::time::Duration::from_millis(100));
        0
    }

    pub fn send_event(this: ComponentRef<Self>, event: EventThreadData){
        'test: loop {
            let entity = match this.try_lock() {
                Ok(ent) => ent,
                Err(err) => continue 'test
            };
            let p_recv = entity.thread_reciever.clone();
            'test2: loop{
                let mut recv = match p_recv.try_lock() {
                    Ok(re) => re,
                    Err(err) => continue 'test2
                };
                match event {
                    EventThreadData::KillEvent() => {
                        recv.clear();
                        recv.push(event.clone());
                    },
                    _ => recv.push(event.clone())
                };
                break 'test;
            }
        }
    }
}

pub struct EntityParams {
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

pub mod entity_event{
    
    use std::collections::HashMap;
    use std::any::*;

    use crate::common::{engine::gamesys::BaseToAny, vertex::*};

    use super::*;

    #[bitmask]
    pub enum EventFlag {
        INIT,
        UPDATE,
        ENTER_AREA,
        LEAVE_AREA,
        STAY_IN_AREA,
        RESPAWN,
        CUSTOM,
        NO_EVENT,

    }

    #[derive(Clone)]
    pub struct Event {
        pub event_flag: EventFlag,
        pub event_name: String,
        pub event_data: EventData,
    }

    #[derive(Clone)]
    pub enum EventDataValue {
        String(String),
        Integer(i32),
        Vector3(Vec3),
        EntityID(EntityID),

    }

    #[derive(Clone)]
    pub enum EventThreadData {
        Event(Event),
        SpecificEvent(EntityID, Event),
        PhysicsEvent(EntityID, EntityID, Event),
        KillEvent()

    }

    #[derive(Clone)]
    pub struct EventData {
        data: HashMap<String, EventDataValue>
    }

    impl EventData {
        pub fn get(&self, name: String) -> Option<&EventDataValue>{
            let value = self.data.get(&name).unwrap();

            value.as_any().downcast_ref()
        }

        pub fn default() -> Self {
            Self { data: HashMap::new() }
        }
    }

    impl Event {
        pub fn init_event() -> Event {
            Event { event_flag: EventFlag::INIT, event_name: String::from("Init"), event_data: EventData::default() }
        }
        pub fn update_event() -> Event {
            Event { event_flag: EventFlag::UPDATE, event_name: String::from("Update"), event_data: EventData::default() }
        }
    }

}

pub struct EntitySystem {
    entities: Box<Vec<ComponentRef<Entity>>>,
    entity_join_handles: Vec<(EntityID, JoinHandle<i32>)>,
    system_status: Arc<Mutex<StatusCode>>,
    thread_reciever: Arc<Mutex<Vec<ThreadData>>>,
    counter: AtomicU32,
}


impl EntitySystem {
    pub fn new() -> EntitySystem {
        let entities = Box::new(Vec::new());

        EntitySystem { 
            entities: entities,
            entity_join_handles: Vec::new(),
            system_status: Arc::new(Mutex::new(StatusCode::INITIALIZE)),
            thread_reciever: Arc::new(Mutex::new(Vec::new())),
            counter: AtomicU32::new(1),

        }
    }

    pub fn init<'a>(this: Arc<Mutex<Self>>){
        unsafe{
            println!("Spawned Entity System!!");
            Self::processing(&mut this.lock().unwrap());
        }
    }

    pub unsafe fn processing(this: &mut Self) -> i32 {
        println!("Enter loop");
        while !Game::isExit() {
            let p_recv = this.thread_reciever.clone();
            let mut recv = p_recv.try_lock();
            if let Ok(ref mut mutex) = recv {
                for th in mutex.as_slice() {
                    let data = th.clone();
                    
                    match data {
                        ThreadData::Empty => todo!(),
                        ThreadData::I32(i) => todo!(),
                        ThreadData::String(s) => todo!(),
                        ThreadData::Vec(vec) => todo!(),
                        ThreadData::Vec3(vec3) => todo!(),
                        ThreadData::Quat(quat) => todo!(),
                        ThreadData::Entity(entity) => {
                            let p_entity = entity.clone();
                            let ent = p_entity.lock().unwrap();
                            let id = ent.entity_id.clone();
                            drop(ent);
                            this.entities.push(entity);
                            let join_handle = std::thread::Builder::new().spawn(|| {Entity::init(p_entity)}).unwrap();
                            this.entity_join_handles.push((id, join_handle));
                            
                        },
                        ThreadData::Status(status) => {
                            let mut sys_status = this.system_status.lock().unwrap();
                            *sys_status = status;
                        }
                        ThreadData::EntityEvent(event) => {
                            
                         for pp_entity in this.entities.to_vec() {
                            let p_entity = pp_entity.clone();
                            Entity::send_event(p_entity, EventThreadData::Event(event.clone()));
                            
                         }
                            
                        }
                        _ => {},
                    }
                }
                mutex.clear();
            }
            drop(recv);

            //let mut entities = this.entities.clone();

            // for p_entity in &*entities {
            //     while let Some(event) = EntitySystem::get_event(this) {
            //         let r_entity = p_entity.try_lock().ok();
            //         if let Some(entity) = r_entity {
            //             let entity_id = entity.entity_id.clone();
            //             drop(entity);
                        
            //             let p_component_sys = this.component_system.clone();
            //             let mut component_sys = p_component_sys.lock().unwrap();
            //             let components = component_sys.entity_get_components(entity_id);
            //             for pp_component in components {
            //                 let p_component = pp_component.clone();
            //                 let component = p_component.lock().unwrap();
            //                 let event_flags = component.get_event_mask().clone();
            //                 if(event_flags.contains(event.event_flag)){
            //                     component.process_event(&event);
            //                 }
            //             }
            //         }
            //     }
            // }
            //println!("dodo");
            this.send_event(Event::update_event());
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        println!("Closing Entity System thread!");

        for p_entity in this.entities.to_vec() {
            let entity = p_entity.lock().unwrap();
            let p_recv = entity.thread_reciever.clone();
            let mut recv = p_recv.lock().unwrap();
            recv.insert(0, EventThreadData::KillEvent());
            drop(recv);
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
        this.entities.clear();
        std::thread::sleep(std::time::Duration::from_millis(100));
        0
    }

    // pub fn get_event(this: &mut Self) -> Option<event::Event> {
    //     let p_events = this.events.clone();
    //     let mut events = match p_events.try_lock().ok() {
    //         Some(e) => e,
    //         None => return None
    //     };
    //     let event = match events.pop(){
    //         Some(e) => Some(e.clone()),
    //         None => None
    //     };
    //     event
    // }

    pub fn send_event(&mut self, event: Event) {
        let p_recv = self.thread_reciever.clone();
        let mut recv = p_recv.lock().unwrap();
        recv.push(ThreadData::EntityEvent(event));
    }

    pub fn is_alive(this: &mut Self) -> bool {
        let p_sys_status = this.system_status.clone();
        let sys_status = p_sys_status.lock().unwrap();
        *sys_status != StatusCode::CLOSE
    }

    pub fn send_status(p_this: ComponentRef<Self>, status: StatusCode) {
        let this = p_this.lock().unwrap();
        let p_stat = this.system_status.clone();
        let mut stat = p_stat.lock().unwrap();
        stat = stat;
    }

    pub fn add_entity(&mut self, params: EntityParams) -> ComponentRef<Entity> {
        let entity = Arc::new(Mutex::new(Entity{
            entity_id: self.counter.fetch_add(1, atomic::Ordering::Relaxed),
            position: params.position,
            rotation: params.rotation,
            scale: params.scale,
            component_system: ComponentRef_new(Vec::<ComponentRef<dyn BaseComponent>>::new()),
            thread_reciever: ComponentRef_new(Vec::new()),
            is_dead: false
        }));
        let p_recv = self.thread_reciever.clone();
        let recv = p_recv.lock();
        recv.unwrap().push(ThreadData::Entity(entity.clone()));
        entity
    }

    pub fn get_entity(&mut self, entity_id: EntityID) -> Option<ComponentRef<Entity>> {
        let p_ents = self.entities.clone();
        let mut entity = None;
        for pp_ent in p_ents.to_vec() {
            let p_ent = pp_ent.clone();
            let ent = p_ent.lock().unwrap();
            if ent.entity_id == entity_id {
                entity = Some(pp_ent.clone());
                break;
            }
        }
        entity
    }

}