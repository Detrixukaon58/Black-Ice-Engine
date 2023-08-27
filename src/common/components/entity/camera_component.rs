#![allow(unused)]
#![allow(non_snake_case)]

use crate::common::{components::{component_system::*, entity::entity_system::*}, *, filesystem::files::*, vertex::*, matrices::*, engine::gamesys::*};

use serde::*;


pub struct CameraComponent {
    projection: MatrixProjection,
    transform: Matrix34,
    pub p_entity: ComponentRef<Entity>,
}

impl Base for CameraComponent {}

impl Reflection for CameraComponent {
    fn register_reflect(&'static self) -> Ptr<Register<>> {
        let mut register = Box::new(Register::new(Box::new(self)));

        Ptr { b: register }
    }
}

impl BaseComponent for CameraComponent {
    fn get_entity(&self) -> ComponentRef<Entity> {
        self.p_entity.clone()
    }

    fn process_event(&mut self, event: &entity_event::Event) {
        use entity_event::EventFlag;
        match event.event_flag {
            EventFlag::INIT => {
                self.init();
            },
            EventFlag::UPDATE => {

            },
            EventFlag::RESPAWN => {

            },
            _ => {

            },
        }
    }

    fn get_event_mask(&self) -> entity_event::EventFlag {
        use entity_event::EventFlag;

        EventFlag::INIT | EventFlag::UPDATE | EventFlag::RESPAWN
    }
}

impl Constructor<CameraComponent> for CameraComponent {
    unsafe fn construct(entity: ComponentRef<Entity>, definition: &ConstructorDefinition) -> Option<ComponentRef<CameraComponent>> {
        Some(ComponentRef_new(Self { projection: MatrixProjection::new(), transform: Matrix34::new(), p_entity: entity.clone() }))
    }
}

impl CameraComponent {
    fn init(&self) {
        
    }
}