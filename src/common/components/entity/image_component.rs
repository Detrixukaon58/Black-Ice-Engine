use crate::common::{components::{component_system::*, entity_system::*}, *, filesystem::files::AssetPath};

use serde::*;

#[derive(Serialize, Deserialize)]
pub struct Image {
    image_file: filesystem::files::AssetPath,
    p_Entity: ComponentRef<Entity>
}

impl Base for Image{}

impl engine::gamesys::Reflection for Image {
    fn registerReflect(&'static self) -> Ptr<Register<>> {
        todo!()
    }
}

impl BaseComponent for Image {
    fn get_entity(&self) -> ComponentRef<Entity> {
        self.p_Entity.clone()
    }

    fn process_event(&self, event: &event::Event) {
        let event_flag = event.event_flag;
        match  event_flag {
            event::EventFlag::INIT => {

            },
            event::EventFlag::UPDATE => {
                println!("This works!!!");
            }
            event::EventFlag::RESPAWN => {

            }
            _ => {}
        }
    }

    fn get_event_mask(&self) -> event::EventFlag {
        use event::EventFlag;
        EventFlag::INIT | EventFlag::UPDATE | EventFlag::RESPAWN
    }
}

impl Constructor<Image> for Image {
    unsafe fn construct(entity: EntityID, definition: &ConstructorDefinition) -> Option<ComponentRef<Image>> {
        let p_ent_sys = Game::get_entity_sys().clone();
        let mut ent_sys = p_ent_sys.lock().unwrap();
        let entity = ent_sys.get_entity(entity).unwrap().clone();
        drop(ent_sys);
        let map = definition.get("image_file").expect("Failed to Build Image").as_object().unwrap();
        let path = String::from(map.get("path").unwrap().as_str().unwrap());
        Some(ComponentRef_new(Self {
            image_file: AssetPath::new(path.clone()),
            p_Entity: entity
        }))

    }
}