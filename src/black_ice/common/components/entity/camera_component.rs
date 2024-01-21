#![allow(unused)]
#![allow(non_snake_case)]

use crate::black_ice::common::{components::{component_system::*, entity::entity_system::*}, *, filesystem::files::*, vertex::*, matrices::*, engine::{gamesys::*, input::InputSystem}, angles::*, transform::Transform};


pub struct CameraComponent {
    projection: MatrixProjection,
    layer: u32,
    camera_id: i32,
    render_texture: Option<filesystem::files::FileSys>,
    view_transform: Matrix34,
    transform: Transform,
    up: Vec3,
    forward: Vec3,
    y: f32,
    pub p_entity: EntityPtr,
    test: Ang3,
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
        let frame_time = event.event_data.get("frame_time".to_string()).unwrap().as_f32().unwrap();
        match event.event_flag {
            EventFlag::INIT => {
                self.init();
                // self.transform.rotate(Quat::euler(Ang3::new(45.0, 0.0, 0.0)));
                self.transform.set_position(Vec3::new(0.0, 0.0, 1.0));
            },
            EventFlag::UPDATE => {

                //self.transform.rotate(Quat::euler(Ang3::new(1.0 * std::time::Duration::from_millis(16).as_secs_f32(), 0.0, 0.0)));
                let (mut cursor_x, mut cursor_y) = InputSystem::get_cursor();
                self.test = Ang3::new(self.test.y + cursor_x.change(), self.test.p + (cursor_y.change() * self.test.y.to_radians().cos()), self.test.r + (cursor_y.change() * self.test.y.to_radians().sin()));
                unsafe {
                    let previous_rotation = self.transform.rotation.to_euler();
                    self.transform.set_rotation(Quat::euler(
                        // Ang3::new(
                        //     -cursor_x.get_position(),
                        //     -cursor_y.get_position() * (-cursor_x.get_position()).to_radians().cos(), 
                        //     -cursor_y.get_position() * (-cursor_x.get_position()).to_radians().sin(), 
                        // )
                        self.test
                    ));
                    
                }
                self.look_at(
                    self.p_entity.get_world_tm() * self.transform.get_world_tm() * Vec3::new(0.0, 0.0, 0.0), 
                    self.p_entity.get_world_tm() * self.transform.get_world_tm() * self.forward
                );
                
                // println!("{}",  self.transform.get_world_tm());
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
        Some(ComponentRef_new(Self { 
            projection: MatrixProjection::new(), 
            camera_id: 0, p_entity: entity.clone(), 
            layer: definition["layer"].as_u32()?, 
            render_texture: None,
            transform: Transform::new(
                definition["position"].as_vec3()?,
                definition["rotation"].as_quat()?,
                definition["scale"].as_vec3()?,),
            up: definition["up"].as_vec3()?,
            forward: definition["forward"].as_vec3()?,
            y: 0.0,
            view_transform: Matrix34::identity(),
            test: Ang3::new(0.0, 0.0, 0.0),
        }))
    }

    fn default_constuctor_definition() -> ConstructorDefinition {
        std::sync::Arc::new(
            Value::Array( vec![
                    Value::Component("layer".to_string(), std::sync::Arc::new(Value::I32(0))),
                    Value::Component("position".to_string(), std::sync::Arc::new(Value::Vec3(Vec3::new(0.0, 0.0, 0.0)))),
                    Value::Component("rotation".to_string(), std::sync::Arc::new(Value::Quat(Quat::euler(Ang3::new(0.0, 0.0, 0.0))))),
                    Value::Component("scale".to_string(), std::sync::Arc::new(Value::Vec3(Vec3::new(1.0, 1.0, 1.0)))),
                    Value::Component("up".to_string(), std::sync::Arc::new(Value::Vec3(Vec3::new(0.0, 0.0, 1.0)))),
                    Value::Component("forward".to_string(), std::sync::Arc::new(Value::Vec3(Vec3::new(1.0, 0.0, 0.0)))),
            ])
        )
    }
}

impl CameraComponent {
    fn init(&mut self) {
        unsafe{
            
            let p_render_sys = Game::get_render_sys();
            let mut render_sys = p_render_sys.write();
            self.camera_id = render_sys.register_camera(self.layer);
            render_sys.update_camera(self.camera_id, &self.projection, &self.p_entity.get_world_tm(), self.up, self.forward);
            drop(render_sys);
            if self.render_texture.is_some() {
                //render_sys.camera_set_render_texture(self.camera_id, , width, height)
            }
            else{
                let width = GAME.window_x as f32;
                let height = GAME.window_y as f32;
                let depth = 1000.0;
                let ratio = width / height;
                // self.projection.ortho_projection(- width / 2.0, width / 2.0, height/ 2.0, -height/ 2.0, -depth / 2.0, depth / 2.0);
                self.projection.perpective_projection(width/height, 75.0, 0.1, depth);
                // self.projection = MatrixProjection::identity();
            }
            InputSystem::reset_cursor();
        }
    }

    fn update(&mut self) {
        unsafe{
            
            let p_render_sys = Game::get_render_sys();
            let mut render_sys = p_render_sys.write();
            render_sys.update_camera(self.camera_id, &self.projection, &(self.view_transform), self.up, self.forward);
            
            drop(render_sys);
            
        }
    }

    fn look_direction(&mut self, position: Vec3, forward: Vec3) {
        
        let right = self.up.cross(forward);
        let new_up = forward.cross(right);
        
        let y = Vec4::new(
            right.y, new_up.y, forward.y, position.y
        );
        let z = Vec4::new(
            right.z, new_up.z, forward.z, position.z
        );
        let x = Vec4::new(
            right.x, new_up.x, forward.x, position.x
        );

        let our_right = self.up.cross(self.forward);

        self.view_transform.x = x * our_right.x + y * our_right.y + z * our_right.z;
        self.view_transform.y = x * self.up.x + y * self.up.y + z * self.up.z;
        self.view_transform.z = x * self.forward.x + y * self.forward.y + z * self.forward.z;
        
    }

    pub fn look_at(&mut self, from: Vec3, to: Vec3) {
        let forward = (to - from).normalized();
        let up = ((self.p_entity.get_world_tm() * self.transform.get_world_tm() * self.up) - from).normalized();
        let right = up.cross(forward);
        let new_up = forward.cross(right);
        // println!("forward: {forward}, right: {right}, up: {new_up}");
        let y = Vec4::new(
            right.y, new_up.y, forward.y, from.y
        );
        let z = Vec4::new(
            right.z, new_up.z, forward.z, from.z
        );
        let x = Vec4::new(
            right.x, new_up.x, forward.x, from.x
        );

        let our_right = self.up.cross(self.forward).normalized();

        self.view_transform.x = (x * our_right.x);
        self.view_transform.x += (y * our_right.y);
        self.view_transform.x += (z * our_right.z);

        self.view_transform.y = (-x * self.up.x);
        self.view_transform.y += (-y * self.up.y);
        self.view_transform.y += (-z * self.up.z);

        self.view_transform.z = (x * self.forward.x);
        self.view_transform.z += (y * self.forward.y);
        self.view_transform.z += (z * self.forward.z);

    }

    pub fn rotate(&mut self, rotation: Quat)
    {
        
        self.transform.rotate(rotation);
        // println!("{}", self.transform.rotation.to_euler());
    }

    pub fn translate(&mut self, translation: Vec3)
    {
        self.transform.translate(translation);
    }

    pub fn set_rotation(&mut self, rotation: Quat) {
        self.transform.set_rotation(rotation);
    }

    pub fn set_position(&mut self, position: Vec3) {
        self.transform.set_position(position);
    }

    pub fn set_scale(&mut self, scale: Vec3) {
        self.transform.set_scale(scale);
    }

    pub fn get_tm(&self) -> Matrix34 {
        let mut tm = Matrix34::identity();
        tm.translate(self.transform.position);
        tm.rotate(self.transform.rotation);
        tm.scale(self.transform.scale);
        tm
    }

    pub fn from_tm(&mut self, tm: Matrix34) {
        self.transform.position = tm.get_translation();
        self.transform.rotation = tm.get_rotation();
        self.transform.scale = tm.get_scale();
    }
}