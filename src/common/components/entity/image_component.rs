#![allow(unused)]
#![allow(non_snake_case)]

use crate::common::{components::{component_system::*, entity::entity_system::*}, *, filesystem::files::*, vertex::*};

use serde::*;


pub struct Image {
    image_file: filesystem::files::AssetPath,
    count: std::time::SystemTime,
    p_Entity: EntityPtr
}

impl Base for Image{}

impl engine::gamesys::Reflection for Image {
    fn register_reflect(&'static self) -> Ptr<Register<>> {
        let mut register = Box::new(Register::new(Box::new(self)));

        register.addProp(Property { 
            name: Box::new("image_file"), 
            desc: Box::new("Path to image file"), 
            reference: Box::new(&self.image_file), 
            ref_type: std::any::TypeId::of::<AssetPath>() });

        Ptr { b: register }
    }
}

impl BaseComponent for Image {
    fn get_entity(&self) -> EntityPtr {
        self.p_Entity.clone()
    }

    fn process_event(&mut self, event: &entity_event::Event) {
        let event_flag = event.event_flag;
        match  event_flag {
            entity_event::EventFlag::INIT => {

            },
            entity_event::EventFlag::UPDATE => {
                println!("{}", std::time::SystemTime::now().duration_since(self.count).unwrap().as_millis());
                self.count = std::time::SystemTime::now();

                //self.draw();
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
    unsafe fn construct(entity: EntityPtr, definition: &ConstructorDefinition) -> Option<ComponentRef<Image>> {
        let map = definition.get("image_file").expect("Failed to Build Image");
        let path = String::from(map.get("path").unwrap().as_str().unwrap());
        println!("{}", path);
        Some(ComponentRef_new(Self {
            image_file: AssetPath::new(path.clone()),
            count: std::time::SystemTime::now(),
            p_Entity: entity.clone()
        }))

    }

    fn default_constuctor_definition() -> ConstructorDefinition {
        std::sync::Arc::new(
            Value::Component(String::from("image_file"), std::sync::Arc::new(
                Value::Component("path".to_string(), std::sync::Arc::new(Value::String(String::new())))
            )),
            
        )
    }
}

impl Image {
    pub fn draw(&mut self) {
        let mut vertices = vec![
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(1.0, 1.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0)
        ];

        let mut temp = vertices.into_iter().map(|vert| vert.to_buffer()).collect::<Vec<Vertex>>();

        let mut faces = vec![
            (0, 1, 2),
            (0, 2, 3)
        ];

        let tex_cood = vec![
            [0.0, 0.0],
            [1.0, 0.0],
            [1.0, 1.0],
            [0.0, 1.0]
        ];
        
        
        let mut image_file = self.image_file.open_as_file();
        
        let image = image_file.read();

        let mut image_bytes = image.as_bytes();

        let image_idat = imagine::png::PngRawChunkIter::new(&image_bytes).enumerate();
        unsafe{
            let mut p_render_sys = Game::get_render_sys().clone();
            let mut render_sys = p_render_sys.write();
            render_sys.quick_redner(temp, faces, tex_cood, image_idat);
        }
    }
}