// TODO: Make an entity registration system to allow for components to be registered to an entity

use std::{any::*, thread::JoinHandle, collections::*, sync::{*, atomic::AtomicU32}, future::*, pin::*};
use serde::*;
use bitmask_enum::*;

use crate::common::{engine::{gamesys::*, threading::ThreadData}, vertex::*, angles::*, components::{component_system::*, entity}};

pub type EntityID = u32;

#[derive(Serialize, Deserialize)]
pub struct Entity {
    pub entity_id: EntityID,
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3
}

unsafe impl Sync for Entity {}

impl Base for Entity{}

impl Reflection for Entity{
    fn registerReflect(&'static self) -> Ptr<Register<>> {
        let mut registration:  Box<Register> = Box::new(Register::new(Box::new(self)));

        registration.addProp(Property { 
            name: Box::new("position"), 
            desc: Box::new("position of the Entity"), 
            reference: Box::new(&self.position), 
            refType: TypeId::of::<Vec3>() 
        });
        registration.addProp(Property { 
            name: Box::new("rotation"), 
            desc: Box::new("rotation of the Entity"), 
            reference: Box::new(&self.rotation), 
            refType: TypeId::of::<Quat>() 
        });
        registration.addProp(Property { 
            name: Box::new("scale"), 
            desc: Box::new("scale of the Entity"), 
            reference: Box::new(&self.scale), 
            refType: TypeId::of::<Vec3>() 
        });

        return Ptr {b: registration};
    }
}

impl Entity {
    pub fn add_component<T>(&mut self, definition: ConstructorDefinition) -> ComponentRef<T> where T: BaseComponent + Constructor<T> {
        unsafe{
            let p_entity_sys = Game::get_entity_sys().clone();
            let mut entity_sys = p_entity_sys.lock().unwrap();
            let component = T::construct(self.entity_id, &definition).unwrap();
            entity_sys.entity_add_component(self.entity_id, component.clone());
            component
        }
    }
}

pub struct EntityParams {
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

pub mod event{
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

}

pub struct EntitySystem {
    entities: Box<Vec<ComponentRef<Entity>>>,
    events: ComponentRef<Vec<event::Event>>,
    system_status: Arc<Mutex<StatusCode>>,
    thread_reciever: Arc<Mutex<Vec<ThreadData>>>,
    counter: AtomicU32,
    component_system: Arc<Mutex<ComponentSystem>>,
}


impl EntitySystem {
    pub fn new() -> EntitySystem {
        let entities = Box::new(Vec::new());

        EntitySystem { 
            entities: entities,
            events: ComponentRef_new(Vec::new()),
            system_status: Arc::new(Mutex::new(StatusCode::INITIALIZE)),
            thread_reciever: Arc::new(Mutex::new(Vec::new())),
            counter: AtomicU32::new(1),
            component_system: Arc::new(Mutex::new(ComponentSystem::new())),

        }
    }

    pub fn init<'a>(this: Arc<Mutex<Self>>){
        unsafe{
            println!("Spawned Entity System!!");
            Self::processing(&mut this.lock().unwrap());
        }
    }

    pub unsafe fn processing<'a>(this: &'a mut Self) -> i32 {

        'run: loop {
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
                            this.entities.push(entity);
                        },
                        ThreadData::Component(id, comp) => {
                            let mut comp_sys = this.component_system.lock().unwrap();
                            comp_sys.entity_add_component(id, comp)
                        }
                        ThreadData::Status(status) => {
                            let mut sys_status = this.system_status.lock().unwrap();
                            *sys_status = status;
                        }
                        _ => {},
                    }
                }
                mutex.clear();
            }
            drop(recv);
            if Game::isExit() {
                break 'run;
            }

            let mut entities = this.entities.clone();

            for p_entity in &*entities {
                while let Some(event) = EntitySystem::get_event(this) {
                    let entity = p_entity.lock().unwrap();
                    let entity_id = entity.entity_id.clone();
                    drop(entity);
                    let p_component_sys = this.component_system.clone();
                    let mut component_sys = p_component_sys.lock().unwrap();
                    let components = component_sys.entity_get_components(entity_id);
                    for pp_component in components {
                        let p_component = pp_component.clone();
                        let component = p_component.lock().unwrap();
                        let event_flags = component.get_event_mask().clone();
                        if(event_flags.contains(event.event_flag)){
                            component.process_event(&event);
                        }
                    }
                }
            }

            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        println!("Closing Entity Sysem thread!");
        std::thread::sleep(std::time::Duration::from_millis(100));
        0
    }

    pub fn get_event(this: &mut Self) -> Option<event::Event> {
        let p_events = this.events.clone();
        let mut events = match p_events.try_lock().ok() {
            Some(e) => e,
            None => return None
        };
        let event = match events.pop(){
            Some(e) => Some(e.clone()),
            None => None
        };
        event
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

    pub fn entity_add_component(&mut self, entity: EntityID, component: ComponentRef<dyn BaseComponent + Send>){
        let p_recv = self.thread_reciever.clone();
        let mut recv = p_recv.lock().unwrap();
        recv.push(ThreadData::Component(entity, component));
    }

    pub fn add_entity(&mut self, params: EntityParams) -> ComponentRef<Entity> {
        let entity = Arc::new(Mutex::new(Entity{
            entity_id: self.counter.fetch_add(1, atomic::Ordering::Relaxed),
            position: params.position,
            rotation: params.rotation,
            scale: params.scale
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
            if(ent.entity_id == entity_id){
                entity = Some(pp_ent.clone());
                break;
            }
        }
        entity
    }

}