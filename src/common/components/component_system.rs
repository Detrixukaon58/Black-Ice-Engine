// TODO: Implement a component registration system to allow for component allocation for entities
#![allow(unused)]
#![allow(non_snake_case)]
use std::sync::*;

use crate::common::{engine::gamesys::*, components::entity::*};

use super::entity::entity_system::EntityID;

use serde::*;


pub struct ComponentSystem {
    component_register: Box<Vec<(entity_system::EntityID, Vec<ComponentRef<dyn BaseComponent>>)>>,
    constructor_register: Box<Vec<(std::any::TypeId, &'static (dyn Fn() -> Option<&'static dyn Base> + Sync))>>,
}

// TODO: Implement a way of reflecting components (need to complent component system first)
pub type ComponentRef<T> = Arc<Mutex<T>>;

pub fn ComponentRef_new<T>(item: T) -> ComponentRef<T> {
    return Arc::new(Mutex::new(item));
}

pub type ConstructorDefinition = serde_json::Value;

pub trait Constructor<T> where T: Base {
    unsafe fn construct(entity: ComponentRef<entity_system::Entity>, definition: &ConstructorDefinition) -> Option<ComponentRef<T>>;
}

pub trait BaseComponent: Reflection + Send{
    fn get_entity(&self) -> ComponentRef<entity_system::Entity>;
    fn process_event(&mut self, event: &entity_system::entity_event::Event);
    fn get_event_mask(&self) -> entity_system::entity_event::EventFlag;
}

impl ComponentSystem {

    pub fn new() -> ComponentSystem {
        ComponentSystem { 
            component_register: Box::new(Vec::new()),
            constructor_register: Box::new(Vec::new()),
        }
    }

    pub fn entity_add_component(&mut self, entity: EntityID, component: ComponentRef<dyn BaseComponent>){
        println!("Adding component!!");
        let register = self.component_register.as_mut();
        for (entity_id, mut vec) in register.to_vec()
        {
            if entity_id.eq(&entity) {
                vec.push(component);
                return;
            }
        }
        register.push((entity, vec![component]));
        
    }

    pub fn entity_get_components(&mut self, entity: EntityID) -> Vec<ComponentRef<dyn BaseComponent>> {

        let register = self.component_register.to_vec();
        for (entity_id, vec) in register {
            if entity_id.eq(&entity) {
                return vec.clone();
            }
        }

        return vec![];

    }


    pub fn init(&'static self){

        
    }

    pub fn processing(&'static self) -> i32 {



        0
    }
}