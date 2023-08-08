// TODO: Make an entity registration system to allow for components to be registered to an entity

use std::{any::*, thread::JoinHandle, collections::*, sync::*, future::*, pin::*};

use crate::common::{engine::gamesys::*, vertex::*, angles::*, components::component_system::*};

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
}


impl EntitySystem {
    pub fn new() -> EntitySystem {
        let entities = Box::new(Vec::new());

        EntitySystem { 
            entities: entities,
            event: None,
            system_status: Arc::new(Mutex::new(StatusCode::INITIALIZE)),

        }
    }

    pub fn init<'a>(this: Arc<Mutex<Self>>){
        unsafe{
            println!("Spawned Entity System!!");
            Self::processing(&mut this.lock().unwrap());
        }
    }

    pub unsafe fn processing<'a>(this: &'a mut Self) -> i32 {

        while !GAME.isExit() {
            

            std::thread::sleep(std::time::Duration::from_millis(5));
        }

        0
    }

}