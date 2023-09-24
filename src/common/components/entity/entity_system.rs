// TODO: Make an entity registration system to allow for components to be registered to an entity
#![allow(unused)]
use std::{any::*, thread::JoinHandle, collections::*, sync::{Arc, atomic::*}, future::*, pin::*, ops::DerefMut, alloc::Layout};
use bitmask_enum::*;

use crate::common::{engine::{gamesys::*, threading::ThreadData}, vertex::*, angles::*, components::{component_system::{*, self}, entity}, transform::{self, Transform}, matrices::Matrix34};
use parking_lot::*;
use colored::*;

use self::entity_event::*;

pub type EntityID = u32;

pub struct Entity {
    pub entity_id: EntityID,
    transform: transform::Transform,
    component_system: ComponentRef<Vec<ComponentRef<dyn BaseComponent>>>,
    thread_reciever: Arc<Mutex<Vec<EventThreadData>>>,
    count: std::time::SystemTime,
    p_avg: Arc<Mutex<Vec<f32>>>,
}

unsafe impl Sync for Entity {}

impl Base for Entity{}

impl std::ops::Deref for Entity {
    type Target = Entity;
    fn deref(&self) -> &Self::Target {
        self
    }
}

impl DerefMut for Entity {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self
    }
}

// impl Default for Entity {
//     fn default() -> Self {
//         Self { 
//             entity_id: Default::default(),
//             transform: transform::Transform::default(),
//             component_system: ComponentRef_new(Vec::<ComponentRef<dyn BaseComponent>>::new()),
//             thread_reciever: Arc::new(Mutex::new(Vec::new())),
//             is_dead: false
//         }
//     }
// }

impl Reflection for Entity{
    fn register_reflect(&'static self) -> Ptr<Register<>> {
        let mut registration:  Box<Register> = Box::new(Register::new(Box::new(self)));

        // registration.addProp(Property { 
        //     name: Box::new("position"), 
        //     desc: Box::new("position of the Entity"), 
        //     reference: Box::new(&self.position), 
        //     ref_type: TypeId::of::<Vec3>() 
        // });
        // registration.addProp(Property { 
        //     name: Box::new("rotation"), 
        //     desc: Box::new("rotation of the Entity"), 
        //     reference: Box::new(&self.rotation), 
        //     ref_type: TypeId::of::<Quat>() 
        // });
        // registration.addProp(Property { 
        //     name: Box::new("scale"), 
        //     desc: Box::new("scale of the Entity"), 
        //     reference: Box::new(&self.scale), 
        //     ref_type: TypeId::of::<Vec3>() 
        // });

        return Ptr {b: registration};
    }
}

impl Entity {
    pub fn add_component<T>(this: &mut EntityPtr, definition: ConstructorDefinition) -> ComponentRef<T> where T: BaseComponent + Constructor<T> {
        unsafe{

            println!("{}", "Adding component to Entity!!".bright_red());
            let p_entity = this.clone();
            let component = T::construct(p_entity, &definition).unwrap();
            let mut c = component.lock();
            let mut event = Event::init_event();
            event.event_data.add_data("frame_time".to_string(), EventDataValue::Float(1.0));
            c.process_event(&event);
            drop(c);
            'test: loop{
                let entity = match this.try_lock() {
                    Some(ent) => ent,
                    None => continue 'test
                };
                let p_comp_sys = entity.component_system.clone();
                let mut comp_sys = p_comp_sys.lock();
                comp_sys.push(component.clone());
                drop(entity);
                drop(comp_sys);
                break;
            }
            println!("{}", "Added component to Entity!!".bright_red());
            component
        }
    }

    pub fn init<'a>(this: &mut EntityPtr) -> i32 {
        Self::processing(this)
    }

    pub fn collect_component(p_this: ComponentRef<Vec<ComponentRef<dyn BaseComponent>>>, count: usize) -> ComponentRef<dyn BaseComponent> {
        let this = p_this.lock();
        return this[count].clone();
    }

    fn len(p_this: ComponentRef<Vec<ComponentRef<dyn BaseComponent>>>) -> usize {
        let this = p_this.lock();
        return this.len().clone();
    }

    fn processing(p_this: &mut EntityPtr) -> i32 {
        'run: loop{
            let mut this = p_this.lock();
            let p_recv = this.thread_reciever.clone();
            let p_comp_sys = this.component_system.clone();
            let mut count = this.count.clone();
            let p_avg = this.p_avg.clone();
            this.count = std::time::SystemTime::now();
            drop(this);

            let mut avg = p_avg.lock();
            avg.push(1.0 / std::time::SystemTime::now().duration_since(count).unwrap().as_secs_f32());
            let average = avg.iter().sum::<f32>() / avg.len() as f32;
            if avg.len() > 60 {
                avg.remove(0);
            }
            let time = std::time::Duration::from_millis(16).as_secs_f32() - avg.last().unwrap();
            std::thread::sleep(std::time::Duration::from_secs_f32(if time > 0.0 {time} else {0.0}));
            let frame_time = 1.0 / average;
            drop(avg);
            let mut i = 0;
            let mut event = Event::update_event();
            event.event_data.add_data("frame_time".to_string(), EventDataValue::Float(frame_time));
            while i < Self::len(p_comp_sys.clone()){
                let p_component = Self::collect_component(p_comp_sys.clone(), i);
                let mut component = p_component.lock();
                if(component.get_event_mask().contains(event.event_flag)){
                    component.process_event(&event);
                }
                i += 1;
            }
            let mut j = 0;
            while let Some(th) = Entity::get_event(&p_recv, j) {
                let data = th.clone();
                if Entity::check_kill(&p_recv) {
                    // kill it early and quickly!!
                    let comp_sys = p_comp_sys.lock();
                    for p_component in  comp_sys.to_vec(){
                        drop(p_component);
                    }
                    break 'run;
                }
                match data {
                    EventThreadData::Event(mut event) => {
                        let mut i = 0;
                        event.event_data.add_data("frame_time".to_string(), EventDataValue::Float(frame_time));
                        while i < Self::len(p_comp_sys.clone()){
                            let p_component = Self::collect_component(p_comp_sys.clone(), i);
                            let mut component = p_component.lock();
                            if(component.get_event_mask().contains(event.event_flag)){
                                component.process_event(&event);
                            }
                            i += 1;
                        }
                    },
                    EventThreadData::KillEvent() => {
                        let comp_sys = p_comp_sys.lock();
                        for p_component in  comp_sys.to_vec(){
                            drop(p_component);
                        }
                        break 'run;
                    }
                    _ => {},
                }
                j += 1;
            }
            
            
        }
        println!("{}", "Killed Entity".red());
        
        0
    }

    fn get_event(p_recv: &Arc<Mutex<Vec<EventThreadData>>>, i: usize) -> Option<EventThreadData> {
        let recv = p_recv.try_lock();
        match recv {
            Some(mut r) => {
                if i < r.len() {
                    Some(r[i].clone())
                }
                else {
                    None
                }
            }
            None => None
        }
    }

    fn check_kill(p_recv: &Arc<Mutex<Vec<EventThreadData>>>) -> bool {
        let recv = p_recv.try_lock();
        match recv {
            Some(mut r) => {
                r.iter().find(|v| match *v {
                    EventThreadData::KillEvent() => true,
                    _ => false
                }).is_some()
            }
            None => false
        }
    }

    pub fn send_event(this: EntityPtr, event: EventThreadData){
        'test: loop {
            let entity = match this.try_lock() {
                Some(ent) => ent,
                None => continue 'test
            };
            let p_recv = entity.thread_reciever.clone();
            'test2: loop{
                let mut recv = match p_recv.try_lock() {
                    Some(re) => re,
                    None => continue 'test2
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

#[derive(Clone)]
pub struct EntityPtr {
    entity: Arc<Mutex<Entity>>,
}

impl EntityPtr {
    pub fn new(entity_id: EntityID, transform: Transform) -> Self {
        unsafe{
            let layout = Layout::new::<Entity>();
            let entity = ComponentRef_new(Entity{
                entity_id: entity_id,
                transform: transform,
                component_system: ComponentRef_new(Vec::<ComponentRef<dyn BaseComponent>>::new()),
                thread_reciever: ComponentRef_new(Vec::new()),
                count: std::time::SystemTime::now(),
                p_avg: Arc::new(Mutex::new(Vec::new())),
            });
            Self { entity: entity}
        }
    }

    pub fn lock<'a>(&self) -> MutexGuard<'_, Entity> {
        self.entity.lock()
    }

    pub fn try_lock<'a>(&self) -> Option<MutexGuard<'_, Entity>>
    {
        self.entity.try_lock()
    }

    pub fn get_world_tm(&self) -> Matrix34
    {
        let entity = self.entity.lock();
        entity.transform.get_world_tm()
    }

    pub fn set_world_tm(&mut self) {

    }

    pub fn rotate(&mut self, rotation: Quat) {
        let mut entity = self.entity.lock();
        entity.transform.rotate(rotation);
    }

    pub fn translate(&mut self, translation: Vec3) {
        let mut entity = self.entity.lock();
        entity.transform.translate(translation);
    }

    pub fn set_rotaion(&mut self, rotation: Quat) {
        let mut entity = self.entity.lock();
        entity.transform.set_rotation(rotation);
    }

    pub fn set_position(&mut self, position: Vec3) {
        let mut entity = self.entity.lock();
        entity.transform.set_position(position);
    }

    pub fn set_scale(&mut self, scale: Vec3) {
        let mut entity = self.entity.lock();
        entity.transform.set_scale(scale);
    }

    pub fn get_rotation(&self) -> Quat {
        let entity = self.entity.lock();
        entity.transform.rotation.clone()
    }

    pub fn get_position(&self) -> Vec3 {
        let entity = self.entity.lock();
        entity.transform.position.clone()
    }

    pub fn get_scale(&self) -> Vec3 {
        let entity = self.entity.lock();
        entity.transform.scale.clone()
    }

    pub fn get_component<T>(&self) -> Option<ComponentRef<T>> where T: BaseComponent + Constructor<T>{
        let entity = self.entity.lock();
        let p_comp_sys = entity.component_system.clone();
        let mut i = 0;
        while i < Entity::len(p_comp_sys.clone()){
            let p_component = Entity::collect_component(p_comp_sys.clone(), i);
            if(p_component.type_id() == TypeId::of::<ComponentRef<T>>()){
                Some(p_component.clone());
            }
            i += 1;
        }
        None
    }

    pub fn get_or_create_component<T>(&mut self, default: ConstructorDefinition) -> ComponentRef<T> where T: BaseComponent + Constructor<T> {
        match self.get_component::<T>() {
            Some(p) => p,
            None => {
                Entity::add_component(self, default)
            }
        }
    }
    
    pub fn add_component<T>(&mut self, definition: ConstructorDefinition) -> ComponentRef<T> where T: BaseComponent + Constructor<T> {
        Entity::add_component(self, definition)
    }

    pub fn is_locked(&self) -> bool {
        self.entity.is_locked()
    }

}

unsafe impl Sync for EntityPtr {}
unsafe impl Send for EntityPtr {}


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
        Float(f32),
        Double(f64),

    }

    impl EventDataValue {
        pub fn as_f32(&self) -> Option<f32> {
            match self {
                Self::Float(v) => Some(*v),
                _ => None
            }
        }
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

        pub fn add_data(&mut self, name: String, data: EventDataValue) {
            self.data.insert(name, data);
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
    p_entities: Arc<Mutex<Vec<EntityPtr>>>,
    entity_join_handles: Arc<Mutex<Vec<(EntityID, JoinHandle<i32>)>>>,
    system_status: Arc<Mutex<StatusCode>>,
    thread_reciever: Arc<Mutex<Vec<ThreadData>>>,
    counter: AtomicU32,
    ready: bool,
}

impl EntitySystem {
    pub fn new() -> EntitySystem {
        let entities = Arc::new(Mutex::new(Vec::<EntityPtr>::new()));

        EntitySystem { 
            p_entities: entities,
            entity_join_handles:  Arc::new(Mutex::new(Vec::new())),
            system_status: Arc::new(Mutex::new(StatusCode::INITIALIZE)),
            thread_reciever: Arc::new(Mutex::new(Vec::new())),
            counter: AtomicU32::new(1),
            ready: false,

        }
    }

    pub fn init<'a>(this: Arc<Mutex<Self>>){
        unsafe{
            println!("Spawned Entity System!!");
            Self::processing(this);
        }
    }

    pub unsafe fn processing(p_this: Arc<Mutex<Self>>) -> i32 {
        println!("Enter loop");
        loop {
            let mut this = p_this.lock();
            let ready = this.ready.clone();
            drop(this);
            if ready {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        while !Game::isExit() {
            let mut this = p_this.lock();
            let p_recv = this.thread_reciever.clone();
            let p_entities = this.p_entities.clone();
            let p_ent_join = this.entity_join_handles.clone();
            let system_status = this.system_status.clone();
            drop(this);
            let mut recv = p_recv.try_lock();
            if let Some(ref mut mutex) = recv {
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
                            let mut p_entity = entity.clone();
                            let ent = p_entity.lock();
                            let id = ent.entity_id.clone();
                            drop(ent);
                            let mut entities = p_entities.lock();
                            let mut entity_join_handles = p_ent_join.lock();
                            entities.push(entity);
                            let join_handle = std::thread::Builder::new().name(format!("Entity_{}", id.clone())).spawn(move || {Entity::init(&mut p_entity)}).unwrap();
                            entity_join_handles.push((id, join_handle));
                            
                        },
                        ThreadData::Status(status) => {
                            let mut sys_status = system_status.lock();
                            *sys_status = status;
                        }
                        ThreadData::EntityEvent(event) => {
                            let mut entities = p_entities.lock();
                            for pp_entity in entities.to_vec() {
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
            //this.send_event(Event::update_event());
            // std::thread::sleep(std::time::Duration::from_millis(5));
        }
        println!("Closing Entity System thread!");
        let mut this = p_this.lock();
        let mut entities = this.p_entities.lock();
        for p_entity in entities.to_vec() {
            let entity = p_entity.lock();
            let p_recv = entity.thread_reciever.clone();
            let mut recv = p_recv.lock();
            recv.insert(0, EventThreadData::KillEvent());
            drop(recv);
            
        }
        let entity_join_handles = this.entity_join_handles.lock();
        for handle in entity_join_handles.as_slice() {
            while !handle.1.is_finished() {
                std::thread::sleep(std::time::Duration::from_millis(1));
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
        entities.clear();
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

    pub fn start(p_this: Arc<Mutex<Self>>) {
        let mut this = p_this.lock();
        println!("{}", "Starting Entity Thread!!".yellow());
        this.ready = true;
    }

    pub fn send_event(&mut self, event: Event) {
        let p_recv = self.thread_reciever.clone();
        let mut recv = p_recv.lock();
        recv.push(ThreadData::EntityEvent(event));
    }

    pub fn is_alive(this: &mut Self) -> bool {
        let p_sys_status = this.system_status.clone();
        let sys_status = p_sys_status.lock();
        *sys_status != StatusCode::CLOSE
    }

    pub fn send_status(p_this: ComponentRef<Self>, status: StatusCode) {
        let this = p_this.lock();
        let p_stat = this.system_status.clone();
        let mut stat = p_stat.lock();
        stat = stat;
    }

    pub fn add_entity(&mut self, params: EntityParams) -> EntityPtr {
        let id = self.counter.fetch_add(1, Ordering::Relaxed);
        let mut trans = transform::Transform::new(params.position, params.rotation, params.scale);
        use transform::TransformSetEntity;
        trans.set_entity(id.clone());
        let entity = EntityPtr::new(
            id,
            trans
        );
        let p_recv = self.thread_reciever.clone();
        let mut recv = p_recv.lock();
        recv.push(ThreadData::Entity(entity.clone()));
        entity
    }

    pub fn get_entity(&mut self, entity_id: EntityID) -> Option<EntityPtr> {
        let p_ents = self.p_entities.clone();
        let ents = p_ents.lock();
        let mut entity = None;
        for pp_ent in ents.to_vec() {
            let p_ent = pp_ent.clone();
            let ent = p_ent.lock();
            if ent.entity_id == entity_id {
                entity = Some(pp_ent.clone());
                break;
            }
        }
        entity
    }

}