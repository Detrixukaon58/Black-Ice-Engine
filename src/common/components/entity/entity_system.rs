// TODO: Make an entity registration system to allow for components to be registered to an entity

use std::{any::*, thread::JoinHandle, collections::*, sync::{*, atomic::AtomicU32}, future::*, pin::*};

use crate::common::{engine::{gamesys::*, threading::ThreadData}, vertex::*, angles::*, components::component_system::*};

pub type EntityID = u32;


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
            let component = T::construct(&definition).unwrap();
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

mod event{
    use std::collections::HashMap;
    use std::any::*;

    use crate::common::{engine::gamesys::BaseToAny, vertex::*};

    use super::*;

    pub enum EventCode {
        INIT,
        UPDATE,
        ENTER_AREA,
        LEAVE_AREA,
        STAY_IN_AREA,
        RESPAWN,
        CUSTOM,

    }

    pub struct Event {
        pub event_code: EventCode,
        pub event_name: String,
        pub event_data: EventData,
    }

    enum EventDataValue {
        String(String),
        Integer(i32),
        Vector3(Vec3),
        EntityID(EntityID),

    }

    pub struct EventData {
        data: HashMap<String, EventDataValue>
    }

    impl EventData {
        pub fn get(&self, name: String) -> Option<&EventDataValue>{
            let value = self.data.get(&name).unwrap();

            value.as_any().downcast_ref()
        }
    }

}

pub struct EntitySystem {
    entities: Box<Vec<ComponentRef<Entity>>>,
    event: Option<event::Event>,
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
            event: None,
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

        while !Game::isExit() {
            let mut recv = this.thread_reciever.try_lock();
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
                    }
                }
                mutex.clear();
            }

            std::thread::sleep(std::time::Duration::from_millis(5));
        }

        0
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

}