#![cfg(feature = "opengl")]
#![allow(unused)]

use std::{sync::Arc, mem::{size_of_val, size_of}, f32::consts::PI};


use colored::Colorize;
use gl46::*;
use sdl2::video::GLContext;
use winit::dpi::Pixel;
use crate::common::{vertex::*, *, materials::Shader, matrices::*, angles::QuatConstructor};
use parking_lot::*;
use super::pipeline::Pipeline;

pub struct SdlGlContext(GLContext);

unsafe impl Send for SdlGlContext{}
unsafe impl Sync for SdlGlContext{}

// Try to find a way to implement a graphics pipeline for opengl that will be similar to Vulkan's and gles!!

pub struct DriverValues {
    pub gl_context: Option<SdlGlContext>,
    pub shader_stages: Vec<(String, u32, u32)>,
    pub gl: Option<GlFns>,
    pub elem_buffer: u32,
    pub vao: u32,
    pub vbo: u32,
    pub frame_buffer: u32,
    pub shader_program: u32,
    
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
    fn render(th: Arc<Mutex<Self>>) -> i32;
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
            
            gl.GenVertexArrays(1, &mut driver.vao);
            assert_ne!(driver.vao, 0);

            gl.GenBuffers(1, &mut driver.vbo);
            gl.GenBuffers(1, &mut driver.elem_buffer);
            assert_ne!(driver.vbo, 0);
            assert_ne!(driver.elem_buffer, 0);
            driver.shader_program = gl.CreateProgram();

            drop(gl);
            drop(driver);
            drop(p_driver);
            pipeline.is_init = true;
        }
        0
    }

    fn render(th: Arc<Mutex<Self>>) -> i32 {
        unsafe {
            
            let mut pipeline = th.lock();
            let mut p_driver = pipeline.driver.lock();
            let driver = p_driver.as_mut().unwrap();
            let mut vao = driver.vao;
            let mut elem_buffer = driver.elem_buffer;
            driver.gl.as_ref().unwrap().Clear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT);
            for p_camera in &pipeline.cameras{
                let camera = p_camera.lock();
                if camera.render_texture.is_some() {
                    driver.gl.as_ref().unwrap().BindFramebuffer(GL_FRAMEBUFFER, camera.render_texture.as_ref().unwrap().inner);
                    driver.gl.as_ref().unwrap().Viewport(0,0, camera.render_texture.as_ref().unwrap().width, camera.render_texture.as_ref().unwrap().height);

                }
                else{
                    driver.gl.as_ref().unwrap().Viewport(0, 0, GAME.window_x as i32, GAME.window_y as i32);
                }
                // println!("{}", pipeline.meshs.len());
                for p_mesh in &pipeline.meshs {
                    let mut mesh = p_mesh.lock();
                    
                    // println!("{}", (camera.projection * camera.transform * mesh.transform * Vec4::new(50.0, 50.0, 0.0, 0.0)));
                    
                    for p_mesh_object in &mesh.meshes {
                        let mesh_object = p_mesh_object.lock();
                        // for vert in &mesh_object.verts {
                        //     println!("{}", (camera.projection * camera.transform * mesh.transform * Vec4::new(vert.x, vert.y, vert.z, 1.0)));
                        // }
                        if !mesh_object.normals.is_empty() {
                            if !mesh_object.texture_coord.is_empty(){
                                let mut buffer = Vec::<(Vec3, Vec3, (f32, f32))>::new();
                                for i in 0..mesh_object.verts.len() {
                                    let vert = mesh_object.verts[i];
                                    let norm = mesh_object.normals.iter().find(|v| v.0 == i as i16).unwrap();
                                    let texcoord = mesh_object.texture_coord.iter().find(|v| v.0 == i as i16).unwrap();
                                    buffer.push((vert.clone(), norm.1.clone(), texcoord.1.clone()));
                                }
                                DriverValues::create_buffer_vec_norm_tex(driver, &buffer, &mesh_object.faces);
                            }
                            else{
                                let mut buffer = Vec::<(Vec3, Vec3)>::new();
                                for i in 0..mesh_object.verts.len() {
                                    let vert = mesh_object.verts[i];
                                    let norm = mesh_object.normals.iter().find(|v| v.0 == i as i16).unwrap();
                                    buffer.push((vert.clone(), norm.1.clone()));
                                }
                                DriverValues::create_buffer_vec_norm(driver, &buffer, &mesh_object.faces);
                            }
                        }
                        else {
                            if !mesh_object.texture_coord.is_empty() {
                                let mut buffer = Vec::<(Vec3, (f32, f32))>::new();
                                for i in 0..mesh_object.verts.len() {
                                    let vert = mesh_object.verts[i];
                                    let texcoord = mesh_object.texture_coord.iter().find(|v| v.0 == i as i16).unwrap();
                                    buffer.push((vert.clone(), texcoord.1.clone()));
                                }
                                DriverValues::create_buffer_vec_tex(driver, &buffer, &mesh_object.faces);
                            }
                            else {
                                let buffer = mesh_object.verts.clone();
                                DriverValues::create_buffer_vec(driver, &buffer, &mesh_object.faces);
                            }
                        }

                        let shader_name = mesh_object.material.shader.shader_name.clone();

                        let sh = DriverValues::register_shader(driver, shader_name, mesh_object.material.shader.clone());
                        let (name, vertex_shader, fragment_shader) = &driver.shader_stages[sh];
                        //let (name, vertex_shader, fragment_shader) = driver.shader_stages.iter().find(|v| v.0 == shader_name).expect(("Shader Compilation Error!!! Shader is missing or not compiled properly!! shader_name:".to_owned() + shader_name.as_str()).as_str());
                        
                        let gl = driver.gl.as_ref().unwrap();
                        let shader_program = driver.shader_program;
                        gl.AttachShader(shader_program, *vertex_shader);
                        gl.AttachShader(shader_program, *fragment_shader);
                        gl.LinkProgram(shader_program);

                        // add other bufferes
                        // just for now we are going to add a vert output and frag output buffer
                        gl.UseProgram(shader_program);
                        let mut count = 0;
                        gl.GetProgramiv(shader_program, GL_ACTIVE_UNIFORMS, &mut count);
                        for i in 0..count {
                            let mut length = 0;
                            let mut size = 0;
                            let mut type_ = std::mem::zeroed();
                            let mut name: [u8; 16] = std::mem::zeroed();
                            gl.GetActiveUniform(shader_program, i as u32, 16, &mut length, &mut size, &mut type_, name.as_mut_ptr());

                            let nn = &std::str::from_utf8(name.as_slice()).unwrap()[..length as usize];
                            
                            match nn {

                                "_mvp" => {
                                    // model view projection
                                    let mvp = camera.projection;
                                    // println!("{:?}", mvp.to_buffer());
                                    gl.ProgramUniformMatrix4fv(shader_program, i, 1, GL_FALSE.0 as u8, mvp.to_buffer().as_ptr());
                                },
                                "_mv" => {
                                    // model view
                                    let mv = camera.transform * mesh.transform;
                                    // println!("{}", mv);
                                    gl.ProgramUniformMatrix4fv(shader_program, i, 1, GL_FALSE.0 as u8, mv.to_buffer44().as_ptr());
                                }
                                "_norm" => {
                                    // rotation of model view
                                    let mv = camera.transform * mesh.transform;
                                    let q = mv.get_rotation();
                                    let norm = q.to_mat33();
                                    gl.ProgramUniformMatrix3fv(shader_program, i, 1, GL_TRUE.0 as u8, norm.to_buffer().as_ptr());
                                }

                                _ => {
                                    // try to get value similar to it in the shader uniforms!!
                                    let uniform = mesh_object.material.shader_descriptor.iter().find(|v| v.0.eq(nn)).expect(("Failed to find uniform. Maybe something went wrong in the Material creation process?? uniform:".to_owned() + format!("{nn}").as_str()).as_str());
                                    let uniform_value = uniform.1.0.clone();

                                    match *uniform_value {
                                        materials::ShaderType::Integer(v) => {
                                            gl.ProgramUniform1i(shader_program, i, v);
                                            
                                        },
                                        materials::ShaderType::Boolean(v) => {
                                            gl.ProgramUniform1i(shader_program, i, v as i32);
                                        },
                                        materials::ShaderType::UnsignedInteger(v) => {
                                            gl.ProgramUniform1ui(shader_program, i, v);
                                        },
                                        materials::ShaderType::Float(v) => {
                                            gl.ProgramUniform1f(shader_program, i, v);
                                        },
                                        materials::ShaderType::Double(v) => {
                                            gl.ProgramUniform1d(shader_program, i, v);
                                        },
                                        materials::ShaderType::Vec3(v) => {
                                            gl.ProgramUniform3fv(shader_program, i, 1, v.as_ptr());
                                        },
                                        materials::ShaderType::Vec4(v) => {
                                            gl.ProgramUniform4fv(shader_program, i, 1, v.as_ptr());
                                        },
                                        materials::ShaderType::Vec2(v) => {
                                            gl.ProgramUniform2fv(shader_program, i, 1, v.as_ptr());
                                        },
                                        materials::ShaderType::IVec3(v) => {
                                            gl.ProgramUniform3iv(shader_program, i, 1, v.as_ptr());
                                        },
                                        materials::ShaderType::IVec4(v) => {
                                            gl.ProgramUniform4iv(shader_program, i, 1, v.as_ptr());
                                        },
                                        materials::ShaderType::IVec2(v) => {
                                            gl.ProgramUniform3iv(shader_program, i, 1, v.as_ptr());
                                        },
                                        materials::ShaderType::UVec3(v) => {
                                            gl.ProgramUniform3uiv(shader_program, i, 1, v.as_ptr());
                                        },
                                        materials::ShaderType::UVec4(v) => {
                                            gl.ProgramUniform4uiv(shader_program, i, 1, v.as_ptr());
                                        },
                                        materials::ShaderType::UVec2(v) => {
                                            gl.ProgramUniform2uiv(shader_program, i, 1, v.as_ptr());
                                        },
                                        materials::ShaderType::DVec3(v) => {
                                            gl.ProgramUniform3dv(shader_program, i, 1, v.as_ptr());
                                        },
                                        materials::ShaderType::DVec4(v) => {
                                            gl.ProgramUniform3dv(shader_program, i, 1, v.as_ptr());
                                        },
                                        materials::ShaderType::DVec2(v) => {
                                            gl.ProgramUniform3dv(shader_program, i, 1, v.as_ptr());
                                        },
                                        materials::ShaderType::Sampler2D(v, width, height) => {
                                            
                                        },
                                    }
                                }

                            }
                            
                            
                        }
                        
                        
                        
                        
                        
                        // gl.BindVertexArray(vao);
                        // gl.DrawArrays(GL_TRIANGLES, 0, mesh_object.verts.len() as i32);
                        gl.BindBuffer(GL_ELEMENT_ARRAY_BUFFER, elem_buffer);
                        gl.DrawElements(GL_TRIANGLES, (mesh_object.faces.len() * 3) as i32, GL_UNSIGNED_INT, 0 as *const _);

                        
                        
                        gl.DetachShader(shader_program, *vertex_shader);
                        gl.DetachShader(shader_program, *fragment_shader);

                        gl.DeleteBuffers(1, &driver.elem_buffer);
                        gl.DeleteBuffers(1, &driver.vbo);
                        gl.DeleteFramebuffers(1, &driver.frame_buffer);
                        gl.DeleteVertexArrays(1, &driver.vao);

                        gl.DeleteProgram(shader_program);
                    }
                }
                GAME.window.gl_swap_window();
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
            vao: 0,
            vbo: 0,
            elem_buffer: 0,
            frame_buffer: 0,
            shader_program: 0,
        }
    }
}

impl DriverValues {
    pub unsafe fn init_ogl(this: &mut Self) {

        

        this.gl_context = Some(SdlGlContext(GAME.window.gl_create_context().expect("Failed to create Context!!")));
        GAME.window.gl_make_current(&this.gl_context.as_ref().unwrap().0).expect("Failed to set current gl context!!");
        // GAME.window.gl_set_context_to_current().expect("Failed to set current gl context!!");

        this.gl = GlFns::load_from(&|f_name| GAME.video.gl_get_proc_address(std::ffi::CStr::from_ptr(f_name.cast()).to_str().unwrap()).cast() ).ok();
        let gl = this.gl.as_ref().unwrap();
        gl.ClearColor(0.2, 0.3, 0.3, 1.0);
        
        


        GAME.video.gl_set_swap_interval(sdl2::video::SwapInterval::VSync).expect("Failed to set swap interval!!");
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

    pub unsafe fn register_shader(this: &mut Self, shader_name: String, shader: Shader) -> usize {
        use filesystem::files::*;
        let gl = this.gl.as_ref().unwrap();
        
        let mut i = 0;
        for (name, _, _) in &this.shader_stages {
            if *name == shader_name {
                return i;
            }
            i +=1;
        }

        let vertex_shader = gl.CreateShader(GL_VERTEX_SHADER);
        assert_ne!(vertex_shader, 0);
        // Gonna have to compile ourselves!!
        let vert_code = FileSys::include_shaders().expand(std::str::from_utf8(shader.vertex_file.file.as_slice()).unwrap()).unwrap();
        gl.ShaderSource(vertex_shader, 1, &vert_code.as_ptr(), &(vert_code.len() as i32));
        
        gl.CompileShader(vertex_shader);
        // gl.ShaderBinary(1, &vertex_shader, GL_SHADER_BINARY_FORMAT_SPIR_V, shader.vertex_file.code.as_ptr().cast(), shader.vertex_file.code.len() as i32);
        // gl.SpecializeShader(vertex_shader, "main".as_ptr(), 0, 0 as *const u32, 0 as *const u32);

        let mut success = 0;
        gl.GetShaderiv(vertex_shader, GL_COMPILE_STATUS, &mut success);
        if success == 0 {
            let mut v: Vec<u8> = Vec::with_capacity(1024);
            let mut log_len = 0_i32;
            gl.GetShaderInfoLog(vertex_shader, 1024, &mut log_len, v.as_mut_ptr().cast());
            v.set_len(log_len.try_into().unwrap());
            panic!("Fragment Compile Error: {}", String::from_utf8_lossy(&v));
        }
        
        let fragment_shader = gl.CreateShader(GL_FRAGMENT_SHADER);
        assert_ne!(fragment_shader, 0);

        let frag_code = FileSys::include_shaders().expand(std::str::from_utf8(shader.fragment_file.file.as_slice()).unwrap()).unwrap();
        gl.ShaderSource(fragment_shader, 1, &frag_code.as_ptr(), &(frag_code.len() as i32));
        
        gl.CompileShader(fragment_shader);
        // gl.ShaderBinary(1, &fragment_shader, GL_SHADER_BINARY_FORMAT_SPIR_V, shader.fragment_file.code.as_ptr().cast(), shader.fragment_file.code.len() as i32);
        // gl.SpecializeShader(fragment_shader, "main".as_ptr(), 0, 0 as *const u32, 0 as *const u32);

        gl.GetShaderiv(fragment_shader, GL_COMPILE_STATUS, &mut success);
        if success == 0 {
            let mut v: Vec<u8> = Vec::with_capacity(1024);
            let mut log_len = 0_i32;
            gl.GetShaderInfoLog(fragment_shader, 1024, &mut log_len, v.as_mut_ptr().cast());
            v.set_len(log_len.try_into().unwrap());
            panic!("Fragment Compile Error: {}", String::from_utf8_lossy(&v));
        }

        this.shader_stages.push((shader_name, vertex_shader, fragment_shader));
        this.shader_stages.len() - 1
    }

    pub unsafe fn create_graphics_pipeline(this: &mut Self, stage: usize) -> PipelineValues {
        let shader_program = &this.shader_stages[stage];
        PipelineValues {  }
    }

    pub unsafe fn create_buffer_vec_norm_tex(this: &mut Self, vertices: &Vec<(Vec3, Vec3, (f32, f32))>, indices: &Vec<(i16, i16, i16)>) {
        let gl = this.gl.as_ref().unwrap();

        gl.BindVertexArray(this.vao);


        gl.BindBuffer(GL_ARRAY_BUFFER, this.vbo);
        let temp = vertices.iter().map(|f| {
            let v = f.0.to_buffer();
            let n = f.1.to_buffer();
            [v[0], v[1], v[2], n[0], n[1], n[2], f.2.0, f.2.1]
        }).collect::<Vec<[f32; 8]>>();
        let verts = temp.as_slice();

        gl.BufferData(GL_ARRAY_BUFFER, size_of_val(verts) as isize, verts.as_ptr().cast(), GL_STATIC_DRAW);
        gl.VertexAttribPointer(0, 3, GL_FLOAT, GL_FALSE.0 as u8, 8 * size_of::<f32>() as i32, 0 as *const _);
        gl.EnableVertexAttribArray(0);
        gl.VertexAttribPointer(1, 3, GL_FLOAT, GL_TRUE.0 as u8, (8 * size_of::<f32>()) as i32, (3 * size_of::<f32>()) as *const _);
        gl.EnableVertexAttribArray(1);
        gl.VertexAttribPointer(2, 2, GL_FLOAT, GL_FALSE.0 as u8, (8 * size_of::<f32>()) as i32, (6 * size_of::<f32>()) as *const _);
        gl.EnableVertexAttribArray(2);

        let ind = indices.iter().map(|v| [v.0 as u32, v.1 as u32, v.2 as u32]).collect::<Vec<[u32;3]>>().as_slice().concat();


        gl.BindBuffer(GL_ELEMENT_ARRAY_BUFFER, this.elem_buffer);
        gl.BufferData(GL_ELEMENT_ARRAY_BUFFER, (ind.len() * size_of::<f32>()) as isize, ind.as_ptr().cast(), GL_STATIC_DRAW);


        
    }

    pub unsafe fn create_buffer_vec_norm(this: &mut Self, vertices: &Vec<(Vec3, Vec3)>, indices: &Vec<(i16, i16, i16)>) {
        let gl = this.gl.as_ref().unwrap();

        gl.BindVertexArray(this.vao);



        gl.BindBuffer(GL_ARRAY_BUFFER, this.vbo);
        let temp = vertices.iter().map(|f| {
            let v = f.0.to_buffer();
            let n = f.1.to_buffer();
            [v[0], v[1], v[2], n[0], n[1], n[2]]
        }).collect::<Vec<[f32; 6]>>();
        let verts = temp.as_slice();

        gl.BufferData(GL_ARRAY_BUFFER, size_of_val(verts) as isize, verts.as_ptr().cast(), GL_STATIC_DRAW);
        gl.VertexAttribPointer(0, 3, GL_FLOAT, GL_FALSE.0 as u8, 6 * size_of::<f32>() as i32, 0 as *const _);
        gl.EnableVertexAttribArray(0);
        gl.VertexAttribPointer(1, 3, GL_FLOAT, GL_TRUE.0 as u8, (6 * size_of::<f32>()) as i32, (3 * size_of::<f32>()) as *const _);
        gl.EnableVertexAttribArray(1);

        let ind = indices.iter().map(|v| [v.0, v.1, v.2]).collect::<Vec<[i16;3]>>().as_slice().concat();


        gl.BindBuffer(GL_ELEMENT_ARRAY_BUFFER, this.elem_buffer);
        gl.BufferData(GL_ELEMENT_ARRAY_BUFFER, (ind.len() * size_of::<f32>()) as isize, ind.as_ptr().cast(), GL_STATIC_DRAW);


    }

    pub unsafe fn create_buffer_vec_tex(this: &mut Self, vertices: &Vec<(Vec3, (f32, f32))>, indices: &Vec<(i16, i16, i16)>) {
        let gl = this.gl.as_ref().unwrap();


        gl.BindVertexArray(this.vao);


        gl.BindBuffer(GL_ARRAY_BUFFER, this.vbo);
        let temp = vertices.iter().map(|f| {
            let v = f.0.to_buffer();
            [v[0], v[1], v[2], f.1.0, f.1.1]
        }).collect::<Vec<[f32; 5]>>();
        let verts = temp.as_slice();

        gl.BufferData(GL_ARRAY_BUFFER, size_of_val(verts) as isize, verts.as_ptr().cast(), GL_STATIC_DRAW);
        gl.VertexAttribPointer(0, 3, GL_FLOAT, GL_FALSE.0 as u8, 5 * size_of::<f32>() as i32, 0 as *const _);
        gl.EnableVertexAttribArray(0);
        gl.VertexAttribPointer(2, 2, GL_FLOAT, GL_TRUE.0 as u8, (5 * size_of::<f32>()) as i32, (3 * size_of::<f32>()) as *const _);
        gl.EnableVertexAttribArray(2);

        let ind = indices.iter().map(|v| [v.0, v.1, v.2]).collect::<Vec<[i16;3]>>().as_slice().concat();

        gl.BindBuffer(GL_ELEMENT_ARRAY_BUFFER, this.elem_buffer);
        gl.BufferData(GL_ELEMENT_ARRAY_BUFFER, (ind.len() * size_of::<f32>()) as isize, ind.as_ptr().cast(), GL_STATIC_DRAW);

        
    }

    pub unsafe fn create_buffer_vec(this: &mut Self, vertices: &Vec<Vec3>, indices: &Vec<(i16, i16, i16)>) {
        let gl = this.gl.as_ref().unwrap();


        gl.BindVertexArray(this.vao);



        gl.BindBuffer(GL_ARRAY_BUFFER, this.vbo);
        let temp = vertices.iter().map(|f| {
            let v = f.to_buffer();
            [v[0], v[1], v[2]]
        }).collect::<Vec<[f32; 3]>>();
        let verts = temp.as_slice();

        gl.BufferData(GL_ARRAY_BUFFER, size_of_val(verts) as isize, verts.as_ptr().cast(), GL_STATIC_DRAW);
        gl.VertexAttribPointer(0, 3, GL_FLOAT, GL_FALSE.0 as u8, 3 * size_of::<f32>() as i32, 0 as *const _);
        gl.EnableVertexAttribArray(0);

        let ind = indices.iter().map(|v| [v.0, v.1, v.2]).collect::<Vec<[i16;3]>>().as_slice().concat();


        gl.BindBuffer(GL_ELEMENT_ARRAY_BUFFER, this.elem_buffer);
        gl.BufferData(GL_ELEMENT_ARRAY_BUFFER, (ind.len() * size_of::<f32>()) as isize, ind.as_ptr().cast(), GL_STATIC_DRAW);

    }
}

pub struct PipelineValues {

}