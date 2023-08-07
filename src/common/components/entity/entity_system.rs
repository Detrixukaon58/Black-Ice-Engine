// TODO: Make an entity registration system to allow for components to be registered to an entity

use std::any::Any;

use crate::common::{engine::gamesys::*, vertex::*, angles::*};


pub struct Entity {
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
    
}

impl Base for Entity {

}

impl Reflection for Entity {
        fn registerReflect(&'static self) -> Ptr<Register<>> {
        let mut register = Box::new(Register::new(Box::new(self)));
        //register.addProp(Property { name: "mesh", desc: "The Mesh", reference: Box::new(&self.mesh), refType: self.mesh.type_id() });
        
        register.addProp(Property { 
            name: Box::new("position"), 
            desc: Box::new("The position of the entity"), 
            reference: Box::new(&self.position), 
            refType: self.position.type_id(),
        });

        return Ptr {b: register};
    }
}

pub struct EntitySystem {
    entities: Box<Vec<Entity>>
}