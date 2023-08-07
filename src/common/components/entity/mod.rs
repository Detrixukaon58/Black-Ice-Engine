mod meshcomponent;
pub mod entity_system;

use std::any::TypeId;

use crate::common::{vertex::Vec3, angles::Quat, engine::gamesys::*};




pub struct Entity {
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3
}

impl Base for Entity{}

impl Reflection for Entity{
    fn registerReflect(&'static self) -> Ptr<Register<>> {
        let mut registration:  Box<Register> = Box::new(Register::new(Box::new(self)));

        registration.addProp(Property { name: Box::new("position"), desc: Box::new("position of the Entity"), reference: Box::new(&self.position), refType: TypeId::of::<Vec3>() });
        registration.addProp(Property { name: Box::new("rotation"), desc: Box::new("rotation of the Entity"), reference: Box::new(&self.rotation), refType: TypeId::of::<Quat>() });
        registration.addProp(Property { name: Box::new("scale"), desc: Box::new("scale of the Entity"), reference: Box::new(&self.scale), refType: TypeId::of::<Vec3>() });

        return Ptr {b: registration};
    }
}