#![cfg(feature = "opengl")]
#![allow(unused)]

use std::{sync::Arc, mem::{size_of_val, size_of}, f32::consts::PI, fs::File, collections::HashMap};


use colored::Colorize;
use gl46::*;
use sdl2::video::GLContext;
use crate::black_ice::common::{vertex::*, *, materials::*, matrices::*, angles::{QuatConstructor, Quat}, mesh::Mesh};
use parking_lot::*;
use super::pipeline::{Pipeline, Camera};

pub struct SdlGlContext(GLContext);

unsafe impl Send for SdlGlContext{}
unsafe impl Sync for SdlGlContext{}

// Try to find a way to implement a graphics pipeline for opengl that will be similar to Vulkan's and gles!!


#[derive(Clone)]
pub struct CameraDriver {
    camera: Arc<Mutex<Camera>>,

}

impl CameraDriver {
    pub fn lock(&self) -> parking_lot::MutexGuard<Camera>
    {
        self.camera.lock()
    }

    pub fn new(p_camera: Arc<Mutex<Camera>>) -> Self {
        Self { camera: p_camera.clone() }
    }
}

pub struct DriverValues {
    pub gl_context: Option<SdlGlContext>,
    pub shader_stages: Vec<ShaderStage>,
    pub gl: Option<GlFns>,
    
}

#[derive(Clone)]
pub enum TextureType {
    RGB,
    DEPTH
}

#[derive(Clone)]
pub struct RenderTexture {
    inner: u32,
    width: i32,
    height: i32,
    texture_type: TextureType
}

pub trait OGlRender {
    fn init(th: Arc<Mutex<Self>>) -> i32;
    fn render(th: Arc<Mutex<Self>>, p_window: Arc<Mutex<sdl2::video::Window>>, p_video: Arc<Mutex<sdl2::VideoSubsystem>>) -> i32;
}

impl OGlRender for Pipeline {

    fn init(th: Arc<Mutex<Self>>) -> i32 {
        unsafe{

            let mut pipeline = th.lock();
            let mut p_driver = pipeline.driver.lock();
            let driver = p_driver.as_mut().unwrap();
            let gl = driver.gl.as_ref().unwrap();
            // this is just to ensure if we need things done before the render loop, it is done here
            // itterate through shader folder and find shaders that are needed to be compiled

            drop(p_driver);
            pipeline.is_init = true;
        }
        0
    }

    fn render(th: Arc<Mutex<Self>>, p_window: Arc<Mutex<sdl2::video::Window>>, p_video: Arc<Mutex<sdl2::VideoSubsystem>>) -> i32 {
        unsafe {
            
            let mut pipeline = th.lock();
            let cameras = pipeline.cameras.clone();
            let mut p_driver = pipeline.driver.lock();
            let driver = p_driver.as_mut().unwrap();
            for p_camera in &cameras{
    
                driver.gl.as_ref().unwrap().Clear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT);
                let camera = p_camera.lock();
                // let mut render_line = Vec::<(u32, u32, u32, u32, Matrix34, u32, u32, HashMap<std::string::String, (Box<materials::ShaderType>, ShaderDataHint)>, usize)>::new();
                if camera.render_texture.is_some() {
                    driver.gl.as_ref().unwrap().BindFramebuffer(GL_FRAMEBUFFER, camera.render_texture.as_ref().unwrap().inner);
                    driver.gl.as_ref().unwrap().Viewport(0,0, camera.render_texture.as_ref().unwrap().width, camera.render_texture.as_ref().unwrap().height);

                }
                else{
                    driver.gl.as_ref().unwrap().Viewport(0, 0, Env::get_env().window_x as i32, Env::get_env().window_y as i32);
                }
                // println!("{}", pipeline.meshs.len());
                
                
                    let gl = driver.gl.as_ref().unwrap();
                    gl.Enable(GL_DEPTH_TEST);
                    
                    gl.DepthMask(GL_FALSE.0 as u8);
                    gl.DepthFunc(GL_ALWAYS);
                    // add other bufferes
                    // just for now we are going to add a vert output and frag output buffer
                    
                    // for mesh in &mut pipeline.meshes.to_vec(){

                    //     mesh.update_mesh(driver, &camera);
                    //     let gl = driver.gl.as_ref().unwrap();
                    //     mesh.draw(gl, &camera);

                    // }
                    // go through all data_sets of this current pipeline!!
                    let window = p_window.lock();
                    window.gl_swap_window();
            }
        }
        0
    }
}

impl Default for DriverValues {
    fn default() -> Self {
        Self { 
            gl_context: None,
            shader_stages: Vec::new(),
            gl: None,
        }
    }
}

impl DriverValues {
    pub unsafe fn init_ogl(this: &mut Self, window: &sdl2::video::Window, video: &sdl2::VideoSubsystem) {

        this.gl_context = Some(SdlGlContext(window.gl_create_context().expect("Failed to create Context!!")));
        window.gl_make_current(&this.gl_context.as_ref().unwrap().0).expect("Failed to set current gl context!!");
        // GAME.window.gl_set_context_to_current().expect("Failed to set current gl context!!");

        this.gl = GlFns::load_from(&|f_name| video.gl_get_proc_address(std::ffi::CStr::from_ptr(f_name.cast()).to_str().unwrap()).cast() ).ok();
        let gl = this.gl.as_ref().unwrap();
        gl.ClearColor(0.2, 0.3, 0.3, 1.0);
        
        


        video.gl_set_swap_interval(sdl2::video::SwapInterval::VSync).expect("Failed to set swap interval!!");
    }

    pub unsafe fn create_render_texture(this: &mut Self, width: i32, height: i32, texture_type: TextureType) -> RenderTexture
    {
        let gl = this.gl.as_ref().unwrap();
        let mut fb = 0;
        gl.GenFramebuffers(1, &mut fb);
        gl.BindFramebuffer(GL_FRAMEBUFFER, fb);

        let mut rend = 0;
        gl.GenTextures(1, &mut rend);

        gl.BindTexture(GL_TEXTURE_2D, rend);
        gl.TexImage2D(GL_TEXTURE_2D, 0, GL_SRGB.0 as i32, width, height, 0, GL_SRGB, GL_UNSIGNED_BYTE, 0 as *const _);
        gl.TexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_NEAREST.0 as i32);
        gl.TexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_NEAREST.0 as i32);

        gl.FramebufferTexture(GL_FRAMEBUFFER, GL_COLOR_ATTACHMENT0, rend, 0);
        gl.DrawBuffers(1, [GL_COLOR_ATTACHMENT0].as_ptr());

        RenderTexture { inner: rend, width:width, height:height, texture_type:texture_type}

    }

    pub unsafe fn create_graphics_pipeline(this: &mut Self, stage: usize) -> PipelineValues {
        let shader_program = &this.shader_stages[stage];
        PipelineValues {  }
    }

    pub unsafe fn create_buffer_vec_norm_tex(this: &mut Self, verts: &[f32], indices: &[u32]) -> (u32, u32, u32){
        let gl = this.gl.as_ref().unwrap();

        let mut vao = {
                let mut v = 0;
                gl.CreateVertexArrays(1, &mut v);
                assert_ne!(v, 0);
                v
        };
        let mut vbo = {
                let mut v = 0;
                gl.GenBuffers(1, &mut v);
                assert_ne!(v, 0);
                v
        };
        let mut elem_buffer = {
                let mut v = 0;
                gl.GenBuffers(1, &mut v);
                assert_ne!(v, 0);
                v
        };

        gl.BindVertexArray(vao);


        gl.BindBuffer(GL_ARRAY_BUFFER, vbo);
        
        gl.BufferData(GL_ARRAY_BUFFER, size_of_val(verts) as isize, verts.as_ptr().cast(), GL_STATIC_DRAW);
        gl.VertexAttribPointer(0, 3, GL_FLOAT, GL_FALSE.0 as u8, 8 * size_of::<f32>() as i32, 0 as *const _);
        gl.EnableVertexAttribArray(0);
        gl.VertexAttribPointer(1, 3, GL_FLOAT, GL_TRUE.0 as u8, (8 * size_of::<f32>()) as i32, (3 * size_of::<f32>()) as *const _);
        gl.EnableVertexAttribArray(1);
        gl.VertexAttribPointer(2, 2, GL_FLOAT, GL_FALSE.0 as u8, (8 * size_of::<f32>()) as i32, (6 * size_of::<f32>()) as *const _);
        gl.EnableVertexAttribArray(2);



        gl.BindBuffer(GL_ELEMENT_ARRAY_BUFFER, elem_buffer);
        gl.BufferData(GL_ELEMENT_ARRAY_BUFFER, (indices.len() * size_of::<f32>()) as isize, indices.as_ptr().cast(), GL_STATIC_DRAW);

        (vao, vbo, elem_buffer)
        
    }

    pub unsafe fn create_buffer_vec_norm(this: &mut Self, verts: &[f32], indices: &[u32]) -> (u32, u32, u32){
        let gl = this.gl.as_ref().unwrap();

        let mut vao = {
            let mut v = 0;
            gl.CreateVertexArrays(1, &mut v);
            assert_ne!(v, 0);
            v
    };
    let mut vbo = {
            let mut v = 0;
            gl.GenBuffers(1, &mut v);
            assert_ne!(v, 0);
            v
    };
    let mut elem_buffer = {
            let mut v = 0;
            gl.GenBuffers(1, &mut v);
            assert_ne!(v, 0);
            v
    };

        gl.BindVertexArray(vao);


        gl.BindBuffer(GL_ARRAY_BUFFER, vbo);

        gl.BufferData(GL_ARRAY_BUFFER, size_of_val(verts) as isize, verts.as_ptr().cast(), GL_STATIC_DRAW);
        gl.VertexAttribPointer(0, 3, GL_FLOAT, GL_FALSE.0 as u8, 6 * size_of::<f32>() as i32, 0 as *const _);
        gl.EnableVertexAttribArray(0);
        gl.VertexAttribPointer(1, 3, GL_FLOAT, GL_TRUE.0 as u8, (6 * size_of::<f32>()) as i32, (3 * size_of::<f32>()) as *const _);
        gl.EnableVertexAttribArray(1);


        gl.BindBuffer(GL_ELEMENT_ARRAY_BUFFER, elem_buffer);
        gl.BufferData(GL_ELEMENT_ARRAY_BUFFER, (indices.len() * size_of::<f32>()) as isize, indices.as_ptr().cast(), GL_STATIC_DRAW);

        (vao, vbo, elem_buffer)
    }

    pub unsafe fn create_buffer_vec_tex(this: &mut Self, verts: &[f32], indices: &[u32]) -> (u32, u32, u32){
        let gl = this.gl.as_ref().unwrap();

        let mut vao = {
            let mut v = 0;
            gl.CreateVertexArrays(1, &mut v);
            assert_ne!(v, 0);
            v
    };
    let mut vbo = {
            let mut v = 0;
            gl.GenBuffers(1, &mut v);
            assert_ne!(v, 0);
            v
    };
    let mut elem_buffer = {
            let mut v = 0;
            gl.GenBuffers(1, &mut v);
            assert_ne!(v, 0);
            v
    };

        gl.BindVertexArray(vao);


        gl.BufferData(GL_ARRAY_BUFFER, size_of_val(verts) as isize, verts.as_ptr().cast(), GL_STATIC_DRAW);
        gl.VertexAttribPointer(0, 3, GL_FLOAT, GL_FALSE.0 as u8, 5 * size_of::<f32>() as i32, 0 as *const _);
        gl.EnableVertexAttribArray(0);
        gl.VertexAttribPointer(2, 2, GL_FLOAT, GL_TRUE.0 as u8, (5 * size_of::<f32>()) as i32, (3 * size_of::<f32>()) as *const _);
        gl.EnableVertexAttribArray(2);


        gl.BindBuffer(GL_ELEMENT_ARRAY_BUFFER, elem_buffer);
        gl.BufferData(GL_ELEMENT_ARRAY_BUFFER, (indices.len() * size_of::<f32>()) as isize, indices.as_ptr().cast(), GL_STATIC_DRAW);

        (vao, vbo, elem_buffer)
    }

    pub unsafe fn create_buffer_vec(this: &mut Self, verts: &[f32], indices: &[u32]) -> (u32, u32, u32){
        let gl = this.gl.as_ref().unwrap();

        let mut vao = {
            let mut v = 0;
            gl.CreateVertexArrays(1, &mut v);
            assert_ne!(v, 0);
            v
    };
    let mut vbo = {
            let mut v = 0;
            gl.GenBuffers(1, &mut v);
            assert_ne!(v, 0);
            v
    };
    let mut elem_buffer = {
            let mut v = 0;
            gl.GenBuffers(1, &mut v);
            assert_ne!(v, 0);
            v
    };

        gl.BindVertexArray(vao);


        gl.BufferData(GL_ARRAY_BUFFER, size_of_val(verts) as isize, verts.as_ptr().cast(), GL_STATIC_DRAW);
        gl.VertexAttribPointer(0, 3, GL_FLOAT, GL_FALSE.0 as u8, 3 * size_of::<f32>() as i32, 0 as *const _);
        gl.EnableVertexAttribArray(0);


        gl.BindBuffer(GL_ELEMENT_ARRAY_BUFFER, elem_buffer);
        gl.BufferData(GL_ELEMENT_ARRAY_BUFFER, (indices.len() * size_of::<f32>()) as isize, indices.as_ptr().cast(), GL_STATIC_DRAW);


        (vao, vbo, elem_buffer)
    }

    pub unsafe fn create_buffer_vec_singular(this: &mut Self, verts: &[f32]) -> (u32, u32, u32) {
        let gl = this.gl.as_ref().unwrap();

        let mut vao = {
            let mut v = 0;
            gl.CreateVertexArrays(1, &mut v);
            assert_ne!(v, 0);
            v
        };
        let mut vbo = {
                let mut v = 0;
                gl.GenBuffers(1, &mut v);
                assert_ne!(v, 0);
                v
        };
        let mut elem_buffer = {
                let mut v = 0;
                gl.GenBuffers(1, &mut v);
                assert_ne!(v, 0);
                v
        };

            gl.BindVertexArray(vao);


            gl.BufferData(GL_ARRAY_BUFFER, size_of_val(verts) as isize, verts.as_ptr().cast(), GL_STATIC_DRAW);
            gl.VertexAttribPointer(0, 3, GL_FLOAT, GL_FALSE.0 as u8, 3 * size_of::<f32>() as i32, 0 as *const _);
            gl.EnableVertexAttribArray(0);


            gl.BindBuffer(GL_ELEMENT_ARRAY_BUFFER, elem_buffer);

            (vao, vbo, elem_buffer)
    }

}

pub struct PipelineValues {

}