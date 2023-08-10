use std::sync::Weak;

use crate::common::{components::{component_system::*, entity_system::*}, *, filesystem::files::AssetPath};

use serde::*;


pub struct Image {
    image_file: filesystem::files::AssetPath,
    count: i32,
    p_Entity: ComponentRef<Entity>
}

impl Base for Image{}

impl engine::gamesys::Reflection for Image {
    fn registerReflect(&'static self) -> Ptr<Register<>> {
        let mut register = Box::new(Register::new(Box::new(self)));

        register.addProp(Property { 
            name: Box::new("image_file"), 
            desc: Box::new("Path to image file"), 
            reference: Box::new(&self.image_file), 
            refType: std::any::TypeId::of::<AssetPath>() });

        Ptr { b: register }
    }
}

impl BaseComponent for Image {
    fn get_entity(&self) -> ComponentRef<Entity> {
        self.p_Entity.clone()
    }

    fn process_event(&mut self, event: &entity_event::Event) {
        let event_flag = event.event_flag;
        match  event_flag {
            entity_event::EventFlag::INIT => {

            },
            entity_event::EventFlag::UPDATE => {
                println!("{}",self.count);
                self.count += 1;
            }
            entity_event::EventFlag::RESPAWN => {

            }
            _ => {}
        }
    }

    fn get_event_mask(&self) -> entity_event::EventFlag {
        use entity_event::EventFlag;
        EventFlag::INIT | EventFlag::UPDATE | EventFlag::RESPAWN
    }
}

impl Constructor<Image> for Image {
    unsafe fn construct(entity: ComponentRef<Entity>, definition: &ConstructorDefinition) -> Option<ComponentRef<Image>> {
        let map = definition.get("image_file").expect("Failed to Build Image").as_object().unwrap();
        let path = String::from(map.get("path").unwrap().as_str().unwrap());
        Some(ComponentRef_new(Self {
            image_file: AssetPath::new(path.clone()),
            count: 0,
            p_Entity: entity.clone()
        }))

    }
}