#![allow(unused)]
#![allow(non_snake_case)]

use engine::{asset_mgr::AssetManager, asset_types::texture::Texture};

use crate::black_ice::common::{components::{component_system::*, entity::entity_system::*}, *, filesystem::files::*, vertex::*};



pub struct Image {
    texture: Texture,
    p_Entity: EntityPtr
}

impl Base for Image{}

impl engine::gamesys::Reflection for Image {
    fn register_reflect(&'static self) -> Ptr<Register<>> {
        let mut register = Box::new(Register::new(Box::new(self)));

        register.addProp(Property { 
            name: Box::new("texture"), 
            desc: Box::new("Path to image file"), 
            reference: Box::new(&self.texture.asset_path), 
            ref_type: std::any::TypeId::of::<Texture>() });

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
        let value = definition.get("texture").expect("Failed to Build Image");
        let path = value.as_str().expect("Failed to load Texture path!!");
        //println!("{}", path);
        Some(ComponentRef_new(Self {
            texture: AssetManager::load_asset(path),
            p_Entity: entity.clone()
        }))

    }

    fn default_constuctor_definition() -> ConstructorDefinition {
        std::sync::Arc::new(
            Value::Component(String::from("texture"), std::sync::Arc::new( Value::String(String::new()))),
            
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
        
        
        unsafe{
            let mut p_render_sys = Env::get_render_sys().clone();
            let mut render_sys = p_render_sys.write();
            
        }
    }
}