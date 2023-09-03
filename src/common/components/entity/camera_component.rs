#![allow(unused)]
#![allow(non_snake_case)]

use crate::common::{components::{component_system::*, entity::entity_system::*}, *, filesystem::files::*, vertex::*, matrices::*, engine::gamesys::*, angles::*};

use serde::*;


pub struct CameraComponent {
    projection: MatrixProjection,
    layer: u32,
    camera_id: i32,
    render_texture: Option<filesystem::files::FileSys>,
    pub p_entity: EntityPtr,
}

impl Base for CameraComponent {}

impl Reflection for CameraComponent {
    fn register_reflect(&'static self) -> Ptr<Register<>> {
        let mut register = Box::new(Register::new(Box::new(self)));

        Ptr { b: register }
    }
}

impl BaseComponent for CameraComponent {
    fn get_entity(&self) -> EntityPtr {
        self.p_entity.clone()
    }

    fn process_event(&mut self, event: &entity_event::Event) {
        use entity_event::EventFlag;
        match event.event_flag {
            EventFlag::INIT => {
                self.init();
            },
            EventFlag::UPDATE => {
                self.update();
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
    unsafe fn construct(entity: EntityPtr, definition: &ConstructorDefinition) -> Option<ComponentRef<CameraComponent>> {
        Some(ComponentRef_new(Self { projection: MatrixProjection::new(), camera_id: 0, p_entity: entity.clone(), layer: definition["layer"].as_u32()?, render_texture: None}))
    }

    fn default_constuctor_definition() -> ConstructorDefinition {
        std::sync::Arc::new(
            Value::Component("layer".to_string(), std::sync::Arc::new(Value::I32(0)))
        )
    }
}

impl CameraComponent {
    fn init(&mut self) {
        unsafe{
            
            let p_render_sys = Game::get_render_sys();
            let mut render_sys = p_render_sys.write();
            self.camera_id = render_sys.register_camera(self.layer);
            render_sys.update_camera(self.camera_id, &self.projection, &self.p_entity.get_world_tm());

            if self.render_texture.is_some() {
                //render_sys.camera_set_render_texture(self.camera_id, , width, height)
            }
            else{
                let width = GAME.window_x as f32;
                let height = GAME.window_y as f32;
                let depth = 1000.0;
                let ratio = width / height;
                self.projection.ortho_projection(- width / 2.0, width / 2.0, height/ 2.0, -height/ 2.0, -depth / 2.0, depth / 2.0);
                // self.projection = MatrixProjection::identity();
            }
            
        }
    }

    fn update(&mut self) {
        unsafe{
            
            let p_render_sys = Game::get_render_sys();
            let mut render_sys = p_render_sys.write();
            render_sys.update_camera(self.camera_id, &self.projection, &self.p_entity.get_world_tm());

            std::thread::sleep(std::time::Duration::from_millis(5));
        }
    }

    pub fn look_at(&mut self, position: Vec3) {
        let norm = position.normalized();
        let forward = (self.p_entity.get_rotation() * Quat::new(1.0, 0.0, 0.0, 0.0)).as_vec3();
    }
}