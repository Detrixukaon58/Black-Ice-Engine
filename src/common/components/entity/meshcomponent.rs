use std::any::Any;

use crate::common::{mesh::*, engine::gamesys::*, vertex::*, angles::Quat};

// This is a type of pointer that is assigned by the game engine. This means that it must be of trait Reflection

struct MeshComponent {
    mesh: Box<Mesh>,
    scale:Vec3,
    position:Vec3,
    rotation:Quat,
}

impl Base for MeshComponent{}

impl Reflection for MeshComponent{
    fn registerReflect(&'static self) -> Ptr<Register<>> {
        let mut register = Box::new(Register::new(Box::new(self)));
        //register.addProp(Property { name: "mesh", desc: "The Mesh", reference: Box::new(&self.mesh), refType: self.mesh.type_id() });
        
        register.addPointer(Pointer{name: Box::new("mesh"), desc: Box::new("The Mesh Pointer"), reference: self.mesh.registerReflect(), refType: self.mesh.type_id()});

        return Ptr {b: register};
    }
}

