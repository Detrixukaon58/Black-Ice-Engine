#![allow(unused)]
#![allow(non_snake_case)]

use std::any::Any;

use crate::common::{mesh::*, engine::gamesys::*, vertex::*, angles::Quat, components::{component_system::*, entity::entity_system::*}};


// This is a type of pointer that is assigned by the game engine. This means that it must be of trait Reflection

struct MeshComponent {
    mesh: Box<Mesh>,
    scale:Vec3,
    position:Vec3,
    rotation:Quat,
    pub p_Entity: ComponentRef<Entity>,
}

impl Base for MeshComponent{}

impl Reflection for MeshComponent{
    fn register_reflect(&'static self) -> Ptr<Register<>> {
        let mut register = Box::new(Register::new(Box::new(self)));
        //register.addProp(Property { name: "mesh", desc: "The Mesh", reference: Box::new(&self.mesh), refType: self.mesh.type_id() });
        
        //register.addPointer(Pointer{name: Box::new("mesh"), desc: Box::new("The Mesh Pointer"), reference: self.mesh.register_reflect(), ref_type: self.mesh.type_id()});

        return Ptr {b: register};
    }
}

impl Constructor<MeshComponent> for MeshComponent {
    unsafe fn construct(entity: ComponentRef<Entity>, definition: &ConstructorDefinition) -> Option<ComponentRef<MeshComponent>> {
        
        let mesh_file = MeshFile::construct(definition.get("file").expect("Corrupted Mesh File Definition!!").clone());

        Some(ComponentRef_new(MeshComponent {
            mesh: Box::new(mesh_file.as_mesh()), 
            scale: definition.get("scale")?.as_vec3()?, 
            position: definition.get("position")?.as_vec3()?, 
            rotation: definition.get("rotation")?.as_quat()?, 
            p_Entity: entity.clone() 
        }))
    }
}

