#![cfg(feature = "opengl")]
#![allow(unused)]

use core::panic;
use std::{collections::HashMap, f32::consts::PI, ffi::CStr, fs::File, mem::{size_of, size_of_val}, os::raw::c_void, sync::Arc};


use colored::Colorize;
use engine::asset_types::shader_asset::ShaderLang;
use gl46::*;
use image::EncodableLayout;
use sdl2::{video::GLContext, surface};
use crate::black_ice::common::{angles::{QuatConstructor, Quat}, engine::pipeline::RenderPipelineSystem, matrices::*, mesh::Mesh, vertex::*, *};
use parking_lot::*;
use self::engine::asset_types::{shader_asset::ShaderType, materials::*};

use super::pipeline::{Pipeline, Camera, Data};

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
    pub shader_stages: Vec<(String, u32)>,
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
    fn cleanup(th:Arc<Mutex<Self>>);
}

impl OGlRender for Pipeline {

    fn init(th: Arc<Mutex<Self>>) -> i32 {
        unsafe{

            let mut pipeline = th.lock();
            let mut p_driver = pipeline.driver.lock();
            let mut driver = p_driver.as_mut().unwrap();
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
            let mut p_driver = pipeline.driver.clone();
            let mut d = p_driver.lock();
            let mut driver = d.as_mut().unwrap();
            let p_shader_input = pipeline.shader_input.clone();
            drop(pipeline);
            for p_camera in &cameras{
    
                driver.gl.as_ref().unwrap().Clear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT);
                let camera = p_camera.lock();
                // let mut render_line = Vec::<(u32, u32, u32, u32, Matrix34, u32, u32, HashMap<std::string::String, (Box<materials::ShaderType>, ShaderDataHint)>, usize)>::new();
                if camera.render_texture.is_some() {
                    driver.gl.as_ref().unwrap().BindFramebuffer(GL_FRAMEBUFFER, camera.render_texture.as_ref().unwrap().inner);
                    driver.gl.as_ref().unwrap().Viewport(0,0, camera.render_texture.as_ref().unwrap().width, camera.render_texture.as_ref().unwrap().height);

                }
                else{
                    let (window_x, window_y) = Env::get_window_size();
                    driver.gl.as_ref().unwrap().Viewport(0, 0, window_x as i32, window_y as i32);
                }

                let camera_projection = camera.projection;
                let camera_transform = camera.transform;
                drop(camera);
                // println!("{}", pipeline.meshs.len());
                
            
                    // add other bufferes
                    // just for now we are going to add a vert output and frag output buffer
                    
                    // for mesh in &mut pipeline.meshes.to_vec(){

                    //     mesh.update_mesh(driver, &camera);
                    //     let gl = driver.gl.as_ref().unwrap();
                    //     mesh.draw(gl, &camera);

                    // }
                    // go through all data_sets of this current pipeline!!

                    pub enum ConvertedData {
                        FloatSequence(String, u32),
                        IntegerSequence(String, u32),
                        I16Sequence(String, u32),
                        DoubleSequence(String, u32),
                        ImageSequence(String, u32),
                        Surface((u32, u32, u32), u32),
                        Float(String, u32),
                        Integer(String, u32),
                        I16(String, u32),
                        Double(String, u32),
                        VectorBuffer(String, u32),
                        Vector(String, u32),
                        IVectorBuffer(String, u32),
                        IVector(String, u32),
                        DVectorBuffer(String, u32),
                        DVector(String, u32),
                        MeshMatrix(String, u32),
                        Matrix(String, u32)
                    }
                    let mut shader_input = p_shader_input.lock();
                    let mut shader_input_clone = shader_input.clone();
                    drop(shader_input);
                    for data in shader_input_clone{
                        // Now get shaders and produce a shader program
                        let mut converted_stages: Vec<u32> = vec![];
                        let p_pipe = th.clone();
                        let pipe = p_pipe.lock();
                        let mut shader_program: Option<u32> = None;
                        if let Some((prog, v)) = pipe.shaders.get(&data.0.asset_path) {
                            shader_program = Some(prog.clone());
                        }
                        drop(pipe);

                        
                        
                        // register each data
                        let mut input_data = data.1.clone();
                        let mut input_uint: Vec<ConvertedData> = vec![];
                        for d in input_data.as_slice() {
                            let mut mesh_buffer: Option<(u32, u32, u32)> = None;
                            let mut buffer: Option<u32> = None;
                            let mut val: Option<u32> = None;
                            match d {
                                Data::FloatSequence(name, fs) => {

                                },
                                Data::IntegerSequence(name, is) => {

                                },
                                Data::I16Sequence(name, i_16s) => {

                                },
                                Data::DoubleSequence(name, d) => {

                                },
                                Data::ImageSequence(name, img) => {
                                    
                                },
                                Data::Surface(p_surface) => {
                                    //sort out like regular mesh

                                    let mut surface = p_surface.lock();
                                    
                                    if !surface.indices.is_empty() && !surface.normals.is_empty() && !surface.texture_coord.is_empty(){
                                        // convert faces to indices
                                        let indices = {
                                            let mut temp = vec![];
                                            for index in &surface.indices {
                                                temp.push(index.clone() as u32);
                                            }
                                            temp
                                        };

                                        let vertices = {
                                            let mut temp = vec![];
                                            for i in 0..surface.verts.len() {
                                                let vert = surface.verts[i];
                                                let normal:Vec3 = {
                                                    let mut temp_n = Vec3::default();
                                                    for normal in &surface.normals {
                                                        if normal.0 == i as i16 {
                                                            temp_n = normal.1.clone();
                                                        }
                                                    }
                                                    temp_n
                                                };
                                                let tex_coord = {
                                                    let mut temp = (0.0, 0.0);
                                                    for tex in &surface.texture_coord {
                                                        if tex.0 == i as i16 {
                                                            temp = tex.1.clone();
                                                        }
                                                    }
                                                    temp
                                                };
                                                temp.push(vert.x);
                                                temp.push(vert.y);
                                                temp.push(vert.z);
                                                temp.push(normal.x);
                                                temp.push(normal.y);
                                                temp.push(normal.z);
                                                temp.push(tex_coord.0);
                                                temp.push(tex_coord.1);
                                            }
                                            temp
                                        };
                                        input_uint.push(ConvertedData::Surface(DriverValues::create_buffer_vec_norm_tex(driver, vertices.as_slice(), indices.as_slice()), surface.indices.len() as u32));

                                    }

                                
                                },
                                Data::Float(name, f) => {

                                },
                                Data::Integer(name, i) => {

                                },
                                Data::I16(name, i_16) => {

                                },
                                Data::Double(name, d) => {

                                },
                                Data::VectorBuffer(name, vs) => {

                                },
                                Data::Vector(name, v) => {

                                },
                                Data::IVectorBuffer(name, vs) => {

                                },
                                Data::IVector(name, v) => {

                                },
                                Data::DVectorBuffer(name, vs) => {

                                },
                                Data::DVector(name, v) => {

                                },
                                Data::MeshMatrix(name, mat) => {
                                    let location = driver.gl.as_ref().unwrap().GetUniformLocation(shader_program.unwrap(), name.as_ptr());
                                    assert_ne!(location, -1);
                                    driver.gl.as_ref().unwrap().UniformMatrix4fv(location, 1, GL_FALSE.0 as u8, mat.to_buffer44().as_mut_ptr());
                                    let _mvp = camera_projection * camera_transform * *mat;
                                    let _mvp_location = driver.gl.as_ref().unwrap().GetUniformLocation(shader_program.unwrap(), "_mvp".as_ptr());
                                    assert_ne!(_mvp_location, -1);
                                    driver.gl.as_ref().unwrap().UniformMatrix4fv(_mvp_location, 1, GL_FALSE.0 as u8, _mvp.to_buffer().as_mut_ptr());
                                },
                                Data::Matrix(name, mat) => {
                                    let location = driver.gl.as_ref().unwrap().GetUniformLocation(shader_program.unwrap(), name.as_ptr());
                                    assert_ne!(location, -1);
                                    driver.gl.as_ref().unwrap().UniformMatrix4fv(location, 1, GL_FALSE.0 as u8, mat.to_buffer44().as_mut_ptr());
                                }
                            }
                        }
                        let gl = driver.gl.as_ref().unwrap();
                        // lets add the view and projection matrices
                        let _p_location = gl.GetUniformLocation(shader_program.unwrap(), "_p".as_ptr());
                        if _p_location != -1 {
                            // we have a pprojection matrix to add!!
                            gl.UniformMatrix4fv(_p_location, 1, GL_FALSE.0 as u8, camera_projection.to_buffer().as_mut_ptr());
                        }
                        let _v_location = gl.GetUniformLocation(shader_program.unwrap(), "_v".as_ptr());
                        if _v_location != -1 {
                            gl.UniformMatrix4fv(_v_location, 1, GL_FALSE.0 as u8, camera_transform.to_buffer44().as_mut_ptr());
                        }

                        gl.UseProgram(shader_program.unwrap());
                        // now we expose the data to the shader program!!
                        for input in &input_uint {
                            match input {
                                ConvertedData::FloatSequence(_, _) => todo!(),
                                ConvertedData::IntegerSequence(_, _) => todo!(),
                                ConvertedData::I16Sequence(_, _) => todo!(),
                                ConvertedData::DoubleSequence(_, _) => todo!(),
                                ConvertedData::ImageSequence(_, _) => todo!(),
                                ConvertedData::Surface((vao,vbo,ebo),count) => {
                                    
                                    gl.BindBuffer(GL_ELEMENT_ARRAY_BUFFER, *ebo);
                                    gl.DrawElements(GL_TRIANGLES, *count as i32, GL_UNSIGNED_INT, 0 as *const _);
                                },
                                ConvertedData::Float(_, _) => todo!(),
                                ConvertedData::Integer(_, _) => todo!(),
                                ConvertedData::I16(_, _) => todo!(),
                                ConvertedData::Double(_, _) => todo!(),
                                ConvertedData::VectorBuffer(_, _) => todo!(),
                                ConvertedData::Vector(_, _) => todo!(),
                                ConvertedData::IVectorBuffer(_, _) => todo!(),
                                ConvertedData::IVector(_, _) => todo!(),
                                ConvertedData::DVectorBuffer(_, _) => todo!(),
                                ConvertedData::DVector(_, _) => todo!(),
                                ConvertedData::Matrix(name, u) => {

                                },
                                ConvertedData::MeshMatrix(name, u) => {

                                }
                            }
                        }

                        
                        
                        for input in &input_uint {
                            match input {
                                ConvertedData::FloatSequence(_, _) => todo!(),
                                ConvertedData::IntegerSequence(_, _) => todo!(),
                                ConvertedData::I16Sequence(_, _) => todo!(),
                                ConvertedData::DoubleSequence(_, _) => todo!(),
                                ConvertedData::ImageSequence(_, _) => todo!(),
                                ConvertedData::Surface((vao,vbo,ebo),count) => {
                                    
                                    gl.DeleteVertexArrays(1, [*vao].as_ptr());
                                    gl.DeleteBuffers(2, [*vbo,*ebo].as_ptr());
                                },
                                ConvertedData::Float(_, _) => todo!(),
                                ConvertedData::Integer(_, _) => todo!(),
                                ConvertedData::I16(_, _) => todo!(),
                                ConvertedData::Double(_, _) => todo!(),
                                ConvertedData::VectorBuffer(_, _) => todo!(),
                                ConvertedData::Vector(_, _) => todo!(),
                                ConvertedData::IVectorBuffer(_, _) => todo!(),
                                ConvertedData::IVector(_, _) => todo!(),
                                ConvertedData::DVectorBuffer(_, _) => todo!(),
                                ConvertedData::DVector(_, _) => todo!(),
                                ConvertedData::Matrix(name, u) => {
                                    
                                },
                                ConvertedData::MeshMatrix(name, u) => {
                                    
                                }
                            }
                        }
                        // we also need to send the output data to the shader_output!!
                        // TODO!!

                        
                    }
                    let mut shader_input = p_shader_input.lock();
                    shader_input.clear();
                    drop(shader_input);

                    let gl = driver.gl.as_ref().unwrap();
                    gl.Enable(GL_DEPTH_TEST);
                    
                    gl.DepthMask(GL_FALSE.0 as u8);
                    gl.DepthFunc(GL_ALWAYS);

                    let window = p_window.lock();
                    window.gl_swap_window();
            }
        }
        0
    }

    fn cleanup(th:Arc<Mutex<Self>>) {
        
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
        
        // get all extentions and print them
        let mut extentions: Vec<String> = vec![];
        let mut extention_count: i32 = 0;
        gl.GetIntegerv(GL_NUM_EXTENSIONS, &mut extention_count);

        for i in 0..extention_count {
            let st = gl.GetStringi(GL_EXTENSIONS, i as u32);
            let c_str = CStr::from_ptr(st as *const i8);
            let value = String::from(c_str.to_str().unwrap());

            print!("{}\n", value);
            extentions.push(value);
                

        }

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


    // pub unsafe fn create_shader(this: &mut Self, stage: &ShaderStage) -> (String, u32) {
    //     for s in &this.shader_stages {
    //         if s.0.eq(&stage.stage_name) {
    //             return s.clone();
    //         }
    //     }
        
    //     let gl = this.gl.as_ref().unwrap();

    //     let shader_uint = gl.CreateShader(match stage.shader_type {
    //         ShaderType::Fragment => GL_FRAGMENT_SHADER,
    //         ShaderType::Vertex => GL_VERTEX_SHADER,
    //         ShaderType::Compute =>GL_COMPUTE_SHADER,
    //     });
    //     let data = (stage.stage_name.clone(), shader_uint.clone());
    //     this.shader_stages.push(data.clone());
    //     return data;
    // }

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

    pub unsafe fn create_uniform_vec(this: &mut Self, vert:&[f32]) -> u32 {
        let gl = this.gl.as_ref().unwrap();

        


        0
    }

    pub unsafe fn create_uniform_int(this:&mut Self, i:&i32) -> u32 {


        0
    }
}

pub struct PipelineValues {

}