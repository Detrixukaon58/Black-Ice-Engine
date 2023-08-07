// TODO: Implement a component registration system to allow for component allocation for entities

use std::sync::*;

use crate::common::{engine::gamesys::*, components::entity::*};


pub struct ComponentSystem {
    component_register: Option<Box<Vec<(entity_system::EntityID, Vec<ComponentRef<&'static dyn BaseComponent>>)>>>,
    constructor_register: Option<Box<Vec<(std::any::TypeId, &'static (dyn Fn() -> Option<&'static dyn Base> + Sync))>>>,
}

// TODO: Implement a way of reflecting components (need to complent component system first)
pub type ComponentRef<T> = Arc<Mutex<T>>;

pub fn ComponentRef_new<T>(item: T) -> ComponentRef<T> {
    return Arc::new(Mutex::new(item));
}

pub struct ConstructorDefinition {

}

pub trait Constructor<T> where T: Base {
    fn construct(definition: &ConstructorDefinition) -> Option<T>;
}

pub trait BaseComponent: Reflection{
    fn get_entity(&self) -> ComponentRef<entity_system::Entity>;
    fn assign_entity(&mut self);
}

impl ComponentSystem {

    pub fn new() -> ComponentSystem {
        ComponentSystem { 
            component_register: None,
            constructor_register: None,
        }
    }

    pub fn entity_add_component<T>(&mut self, entity: ComponentRef<entity_system::Entity>, definition: &ConstructorDefinition) where T: BaseComponent + Constructor<T>{

        let component = T::construct(definition);

        

    }

    pub fn init(&'static self){

        
    }

    pub fn processing(&'static self) -> i32 {



        0
    }
}