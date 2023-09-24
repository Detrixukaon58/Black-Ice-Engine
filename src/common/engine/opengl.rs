#![cfg(feature = "opengl")]
#![allow(unused)]

use std::{sync::Arc, mem::{size_of_val, size_of}, f32::consts::PI, fs::File, collections::HashMap};


use colored::Colorize;
use gl46::*;
use sdl2::video::GLContext;
use crate::common::{vertex::*, *, materials::{Shader, ShaderDataHint}, matrices::*, angles::{QuatConstructor, Quat}, mesh::Mesh};
use parking_lot::*;
use super::pipeline::{Pipeline, Camera};

pub struct SdlGlContext(GLContext);

unsafe impl Send for SdlGlContext{}
unsafe impl Sync for SdlGlContext{}

// Try to find a way to implement a graphics pipeline for opengl that will be similar to Vulkan's and gles!!

#[derive(Clone)]
pub struct MeshDriver {
    p_mesh: Arc<Mutex<Mesh>>,
    vao: Arc<Mutex<Vec<u32>>>,
    vbo: Arc<Mutex<Vec<u32>>>,
    elem_buffer: Arc<Mutex<Vec<u32>>>,
    shader_programs: Arc<Mutex<Vec<u32>>>,
    vertices: Arc<Mutex<Vec<Vec<f32>>>>,
    indices: Arc<Mutex<Vec<Vec<u32>>>>,
}

impl MeshDriver {
    pub fn draw(&self, gl: &GlFns, camera: &Camera) {
        let p_mesh = self.p_mesh.clone();
        let mesh = p_mesh.lock();
        let vao_buff = self.vao.lock();
        let vbo_buff = self.vbo.lock();
        let shads = self.shader_programs.lock();
        let elems = self.elem_buffer.lock();
        let inds = self.indices.lock();
        for i in 0..(shads.len()) {
            let mesh_object = mesh.meshes[i].lock();
            let vao = vao_buff[i];
            let vbo = vbo_buff[i];
            let elem_buffer = elems[i];
            let shader_program = shads[i];
            let ind = &inds[i];
            unsafe{
                
                gl.UseProgram(shader_program);                
                let mut count = 0;
                gl.GetProgramiv(shader_program, GL_ACTIVE_UNIFORMS, &mut count);
                for i in 0..count {
                    let mut length = 0;
                    let mut size = 0;
                    let mut type_ = std::mem::zeroed();
                    let mut name: [u8; 128] = std::mem::zeroed();
                    gl.GetActiveUniform(shader_program, i as u32, 128, &mut length, &mut size, &mut type_, name.as_mut_ptr());

                    let nn = &std::str::from_utf8(name.as_slice()).unwrap()[..length as usize];
                    
                    match nn {

                        "_p" => {
                            // model view projection
                            
                            // let right = camera.up.cross(camera.forward);
                            let mut p = camera.projection;

                            gl.ProgramUniformMatrix4fv(shader_program, i, 1, GL_FALSE.0 as u8, p.to_buffer().as_ptr());
                        },
                        "_mvp" => {
                            // model view projection

                            let right = camera.up.cross(camera.forward);
                            
                            let mut proj = camera.projection;

                            let mut m = mesh.transform;
                            m.x = -mesh.transform.x * right.x;
                            m.x += -mesh.transform.y * right.y;
                            m.x += -mesh.transform.z * right.z;
                            m.y = -mesh.transform.x * camera.up.x;
                            m.y += -mesh.transform.y * camera.up.y;
                            m.y += -mesh.transform.z * camera.up.z;
                            m.z = -mesh.transform.x * camera.forward.x;
                            m.z += -mesh.transform.y * camera.forward.y;
                            m.z += -mesh.transform.z * camera.forward.z;
                            
                            let mvp = proj * camera.transform * m;
                            gl.ProgramUniformMatrix4fv(shader_program, i, 1, GL_FALSE.0 as u8, mvp.to_buffer().as_ptr());
                        },
                        "_m" => {
                            // model view
                            let right = camera.up.cross(camera.forward);
                            let mut m = mesh.transform;
                            m.x = -mesh.transform.x * right.x;
                            m.x += -mesh.transform.y * right.y;
                            m.x += -mesh.transform.z * right.z;
                            m.y = -mesh.transform.x * camera.up.x;
                            m.y += -mesh.transform.y * camera.up.y;
                            m.y += -mesh.transform.z * camera.up.z;
                            m.z = -mesh.transform.x * camera.forward.x;
                            m.z += -mesh.transform.y * camera.forward.y;
                            m.z += -mesh.transform.z * camera.forward.z;
                            
                            gl.ProgramUniformMatrix4fv(shader_program, i, 1, GL_FALSE.0 as u8, m.to_buffer44().as_ptr());
                        }
                        "_v" => {
                            // model view
                            let v = camera.transform;

                            gl.ProgramUniformMatrix4fv(shader_program, i, 1, GL_FALSE.0 as u8, v.to_buffer44().as_ptr());
                        }
                        "_norm" => {
                            // rotation of model view
                            let right = camera.up.cross(camera.forward);
                            
                            let mut m = mesh.transform;
                            m.x = mesh.transform.x * right.x;
                            m.x += mesh.transform.y * right.y;
                            m.x += mesh.transform.z * right.z;
                            m.y = mesh.transform.x * camera.up.x;
                            m.y += mesh.transform.y * camera.up.y;
                            m.y += mesh.transform.z * camera.up.z;
                            m.z = mesh.transform.x * camera.forward.x;
                            m.z += mesh.transform.y * camera.forward.y;
                            m.z += mesh.transform.z * camera.forward.z;
                            let mv = camera.transform * m;
                            let q = mv.get_rotation();
                            let norm = q.to_mat33();
                            gl.ProgramUniformMatrix3fv(shader_program, i, 1, GL_FALSE.0 as u8, norm.to_buffer().as_ptr());
                        }
                        "nemissa" => {
                            // let file = image::open(APP_DIR.to_owned() + "\\assets\\images\\nemissa_hitomi.png").unwrap();
                            // let mut texture = 0;
                            // gl.GenTextures(1, &mut texture);
                            // gl.BindTexture(GL_TEXTURE_2D, texture);
                            // gl.TexImage2D(GL_TEXTURE_2D, 0, GL_RGB.0 as i32, file.width() as i32, file.height() as i32, 0, GL_RGB, GL_UNSIGNED_BYTE, file.as_bytes().as_ptr().cast());
                            
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
                gl.BindVertexArray(vao);
                gl.BindBuffer(GL_ARRAY_BUFFER, vbo);
                gl.BindBuffer(GL_ELEMENT_ARRAY_BUFFER, elem_buffer);
                let mut buff_size = 0;
                gl.GetBufferParameteriv(GL_ARRAY_BUFFER, GL_BUFFER_SIZE, &mut buff_size);
                gl.DrawElements(GL_TRIANGLES, ind.len() as i32, GL_UNSIGNED_INT, 0 as *const _);
            }
        }
        
    }

    // Usefull for live remeshing
    pub unsafe fn update_mesh(&mut self, driver: &mut DriverValues, camera: &Camera) {
        
        let mesh = self.p_mesh.lock();

        let mut vao_vec = Vec::new();
        let mut vbo_vec = Vec::new();
        let mut elem_buff_vec = Vec::new();
        let mut shader_progs = Vec::new();
        let mut vertices = Vec::<Vec<f32>>::new();
        let mut indices = Vec::<Vec<u32>>::new();
        
        let mut vao_buff = self.vao.lock();
        let mut vbo_buff = self.vbo.lock();
        let mut shads = self.shader_programs.lock();
        let mut elems = self.elem_buffer.lock();
        let mut verts = self.vertices.lock();
        let mut inds = self.indices.lock();

        let right = camera.up.cross(camera.forward);
        for i in 0..mesh.meshes.len() {
            let gl = driver.gl.as_ref().unwrap();
            let mesh_object = mesh.meshes[i].lock();
            let mut vao = match vao_buff.get(i) {
                Some(v) => *v,
                None => {
                    let mut v = 0;
                    gl.CreateVertexArrays(1, &mut v);
                    assert_ne!(v, 0);
                    v
            }
            };
            let mut vbo = match vbo_buff.get(i) {
                Some(v) => *v,
                None => {
                    let mut v = 0;
                    gl.GenBuffers(1, &mut v);
                    assert_ne!(v, 0);
                    v
             }
            };
            let (mut elem_buffer) = match elems.get(i) {
                Some(v) => *v,
                None => {
                    let mut v = 0;
                    gl.GenBuffers(1, &mut v);
                    assert_ne!(v, 0);
                    v
            }
            };
            let mut shader_program = 0;
            unsafe{           
                if !mesh_object.normals.is_empty() {
                    if !mesh_object.texture_coord.is_empty(){
                        let mut buffer = Vec::<(Vec3, Vec3, (f32, f32))>::new();
                        for i in 0..mesh_object.verts.len() {
                            let vert = mesh_object.verts[i];
                            let norm = mesh_object.normals.iter().find(|v| v.0 == i as i16).unwrap();
                            let texcoord = mesh_object.texture_coord.iter().find(|v| v.0 == i as i16).unwrap();
                            buffer.push((vert.clone(), norm.1.clone(), texcoord.1.clone()));
                        }
                        gl.BindVertexArray(vao);


                        gl.BindBuffer(GL_ARRAY_BUFFER, vbo);
                        let temp = buffer.iter().map(|f| {
                            let v = f.0.to_buffer();
                            let n = f.1.to_buffer();
                            vec![
                                v[0], 
                                v[1], 
                                v[2], 
                                n[0], 
                                n[1], 
                                n[2], 
                                f.2.0, 
                                f.2.1
                                ]
                        }).collect::<Vec<Vec<f32>>>();
                        let vertice = temp.iter().flatten().map(|v| v.clone()).collect::<Vec<f32>>();
                        vertices.push(vertice);
                        let verts = vertices[i].as_slice();


                        gl.BufferData(GL_ARRAY_BUFFER, size_of_val(verts) as isize, verts.as_ptr().cast(), GL_STATIC_DRAW);
                        gl.VertexAttribPointer(0, 3, GL_FLOAT, GL_FALSE.0 as u8, 8 * size_of::<f32>() as i32, 0 as *const _);
                        gl.EnableVertexAttribArray(0);
                        gl.VertexAttribPointer(1, 3, GL_FLOAT, GL_TRUE.0 as u8, (8 * size_of::<f32>()) as i32, (3 * size_of::<f32>()) as *const _);
                        gl.EnableVertexAttribArray(1);
                        gl.VertexAttribPointer(2, 2, GL_FLOAT, GL_FALSE.0 as u8, (8 * size_of::<f32>()) as i32, (6 * size_of::<f32>()) as *const _);
                        gl.EnableVertexAttribArray(2);
                        

                        let indice = mesh_object.faces.iter().map(|v| [v.0 as u32, v.1 as u32, v.2 as u32]).collect::<Vec<[u32;3]>>().as_slice().concat();
                        indices.push(indice);
                        let ind = indices[i].as_slice();
                
                        gl.BindBuffer(GL_ELEMENT_ARRAY_BUFFER, elem_buffer);
                        gl.BufferData(GL_ELEMENT_ARRAY_BUFFER, (ind.len() * size_of::<f32>()) as isize, ind.as_ptr().cast(), GL_STATIC_DRAW);

                    }
                    else{
                        let mut buffer = Vec::<(Vec3, Vec3)>::new();
                        for i in 0..mesh_object.verts.len() {
                            let vert = mesh_object.verts[i];
                            let norm = mesh_object.normals.iter().find(|v| v.0 == i as i16).unwrap();
                            buffer.push((vert.clone(), norm.1.clone()));
                        }
                        gl.BindVertexArray(vao);


                        gl.BindBuffer(GL_ARRAY_BUFFER, vbo);
                        let temp = buffer.iter().map(|f| {
                            let v = f.0.to_buffer();
                            let n = f.1.to_buffer();
                            vec![
                                v[0], 
                                v[1], 
                                v[2], 
                                n[0], 
                                n[1], 
                                n[2], 
                                ]
                        }).collect::<Vec<Vec<f32>>>();
                        let vertice = temp.iter().flatten().map(|v| v.clone()).collect::<Vec<f32>>();
                        vertices.push(vertice);
                        let verts = vertices[i].as_slice();
                        
                        gl.BufferData(GL_ARRAY_BUFFER, size_of_val(verts) as isize, verts.as_ptr().cast(), GL_STATIC_DRAW);
                        gl.VertexAttribPointer(0, 3, GL_FLOAT, GL_FALSE.0 as u8, 6 * size_of::<f32>() as i32, 0 as *const _);
                        gl.EnableVertexAttribArray(0);
                        gl.VertexAttribPointer(1, 3, GL_FLOAT, GL_TRUE.0 as u8, (6 * size_of::<f32>()) as i32, (3 * size_of::<f32>()) as *const _);
                        gl.EnableVertexAttribArray(1);
                
                        let indice = mesh_object.faces.iter().map(|v| [v.0 as u32, v.1 as u32, v.2 as u32]).collect::<Vec<[u32;3]>>().as_slice().concat();
                        indices.push(indice);
                        let ind = indices[i].as_slice();
                
                        gl.BindBuffer(GL_ELEMENT_ARRAY_BUFFER, elem_buffer);
                        gl.BufferData(GL_ELEMENT_ARRAY_BUFFER, (ind.len() * size_of::<f32>()) as isize, ind.as_ptr().cast(), GL_STATIC_DRAW);
                        
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
                        gl.BindVertexArray(vao);


                        gl.BindBuffer(GL_ARRAY_BUFFER, vbo);
                        let temp = buffer.iter().map(|f| {
                            let v = f.0.to_buffer();
                            vec![
                                v[0], 
                                v[1], 
                                v[2], 
                                f.1.0, 
                                f.1.1
                                ]
                        }).collect::<Vec<Vec<f32>>>();
                        let vertice = temp.iter().flatten().map(|v| v.clone()).collect::<Vec<f32>>();
                        vertices.push(vertice);
                        let verts = vertices[i].as_slice();
                        
                        gl.BufferData(GL_ARRAY_BUFFER, size_of_val(verts) as isize, verts.as_ptr().cast(), GL_STATIC_DRAW);
                        gl.VertexAttribPointer(0, 3, GL_FLOAT, GL_FALSE.0 as u8, 5 * size_of::<f32>() as i32, 0 as *const _);
                        gl.EnableVertexAttribArray(0);
                        gl.VertexAttribPointer(2, 2, GL_FLOAT, GL_FALSE.0 as u8, (5 * size_of::<f32>()) as i32, (3 * size_of::<f32>()) as *const _);
                        gl.EnableVertexAttribArray(2);
                
                        let indice = mesh_object.faces.iter().map(|v| [v.0 as u32, v.1 as u32, v.2 as u32]).collect::<Vec<[u32;3]>>().as_slice().concat();
                        indices.push(indice);
                        let ind = indices[i].as_slice();
                
                        gl.BindBuffer(GL_ELEMENT_ARRAY_BUFFER, elem_buffer);
                        gl.BufferData(GL_ELEMENT_ARRAY_BUFFER, (ind.len() * size_of::<f32>()) as isize, ind.as_ptr().cast(), GL_STATIC_DRAW);
                       
                        
                    }
                    else {
                        let buffer = mesh_object.verts.clone();
                        gl.BindVertexArray(vao);


                        gl.BindBuffer(GL_ARRAY_BUFFER, vbo);
                        let temp = buffer.iter().map(|f| {
                            let v = f.to_buffer();
                            vec![
                                v[0], 
                                v[1], 
                                v[2], 

                                ]
                        }).collect::<Vec<Vec<f32>>>();
                        let vertice = temp.iter().flatten().map(|v| v.clone()).collect::<Vec<f32>>();
                        vertices.push(vertice);
                        let verts = vertices[i].as_slice();
                        
                        gl.BufferData(GL_ARRAY_BUFFER, size_of_val(verts) as isize, verts.as_ptr().cast(), GL_STATIC_DRAW);
                        gl.VertexAttribPointer(0, 3, GL_FLOAT, GL_FALSE.0 as u8, 3 * size_of::<f32>() as i32, 0 as *const _);
                        gl.EnableVertexAttribArray(0);
                
                        let indice = mesh_object.faces.iter().map(|v| [v.0 as u32, v.1 as u32, v.2 as u32]).collect::<Vec<[u32;3]>>().as_slice().concat();
                        indices.push(indice);
                        let ind = indices[i].as_slice();
                
                        gl.BindBuffer(GL_ELEMENT_ARRAY_BUFFER, elem_buffer);
                        gl.BufferData(GL_ELEMENT_ARRAY_BUFFER, (ind.len() * size_of::<f32>()) as isize, ind.as_ptr().cast(), GL_STATIC_DRAW);
                    }
                }
            }
            let shader_name = mesh_object.material.shader.shader_name.clone();

            let sh = DriverValues::register_shader(driver, shader_name, mesh_object.material.shader.clone());
            let (name, vertex_shader, fragment_shader) = &driver.shader_stages[sh];
            let gl = driver.gl.as_ref().unwrap();
            shader_program = match shads.get(i) {
                Some(v) => *v,
                None => {
                    let v = gl.CreateProgram();
                    v
                }
            };
            gl.AttachShader(shader_program, *vertex_shader);
            gl.AttachShader(shader_program, *fragment_shader);
            gl.LinkProgram(shader_program);
            vao_vec.push(vao);
            vbo_vec.push(vbo);
            elem_buff_vec.push(elem_buffer);
            shader_progs.push(shader_program);
        }
        drop(mesh);
        *vao_buff = vao_vec;
        *vbo_buff = vbo_vec;
        *elems = elem_buff_vec;
        *shads = shader_progs;
        *verts = vertices;
        *inds = indices;

    }

    pub fn new(p_driver: Arc<Mutex<Option<DriverValues>>>, p_mesh: Arc<Mutex<Mesh>>) -> Self {
        let mut dd = p_driver.lock();
        let mut driver = dd.as_mut().unwrap();
        let mesh = p_mesh.lock();
        let mut vao_vec = Vec::new();
        let mut vbo_vec = Vec::new();
        let mut elem_buff_vec = Vec::new();
        let mut shader_progs = Vec::new();
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        for p_mesh_object in &mesh.meshes {
            let mesh_object = p_mesh_object.lock();
            let mut vao = 0;
            let mut vbo = 0;
            let mut elem_buffer = 0;
            let mut shader_program = 0;
            unsafe{           
                if !mesh_object.normals.is_empty() {
                    if !mesh_object.texture_coord.is_empty(){
                        let mut buffer = Vec::<(Vec3, Vec3, (f32, f32))>::new();
                        for i in 0..mesh_object.verts.len() {
                            let vert = mesh_object.verts[i];
                            let norm = mesh_object.normals.iter().find(|v| v.0 == i as i16).unwrap();
                            let texcoord = mesh_object.texture_coord.iter().find(|v| v.0 == i as i16).unwrap();
                            buffer.push((vert.clone(), norm.1.clone(), texcoord.1.clone()));
                        }
                        let temp = buffer.iter().map(|f| {
                            let v = f.0.to_buffer();
                            let n = f.1.to_buffer();
                            vec![
                                v[0], 
                                v[1], 
                                v[2], 
                                n[0], 
                                n[1], 
                                n[2], 
                                f.2.0, 
                                f.2.1
                                ]
                        }).collect::<Vec<Vec<f32>>>();
                        let vertice = temp.iter().flatten().map(|v| v.clone()).collect::<Vec<f32>>();
                        vertices.push(vertice);
                        let verts = vertices.last().unwrap().as_slice();
                        let indice = mesh_object.faces.iter().map(|v| [v.0 as u32, v.1 as u32, v.2 as u32]).collect::<Vec<[u32;3]>>().as_slice().concat();
                        indices.push(indice);
                        let ind = indices.last().unwrap().as_slice();
                        (vao, vbo, elem_buffer) = DriverValues::create_buffer_vec_norm_tex(&mut driver, verts, ind);
                    }
                    else{
                        let mut buffer = Vec::<(Vec3, Vec3)>::new();
                        for i in 0..mesh_object.verts.len() {
                            let vert = mesh_object.verts[i];
                            let norm = mesh_object.normals.iter().find(|v| v.0 == i as i16).unwrap();
                            buffer.push((vert.clone(), norm.1.clone()));
                        }
                        let temp = buffer.iter().map(|f| {
                            let v = f.0.to_buffer();
                            let n = f.1.to_buffer();
                            vec![
                                v[0], 
                                v[1], 
                                v[2], 
                                n[0], 
                                n[1], 
                                n[2], 
                                ]
                        }).collect::<Vec<Vec<f32>>>();
                        let vertice = temp.iter().flatten().map(|v| v.clone()).collect::<Vec<f32>>();
                        vertices.push(vertice);
                        let verts = vertices.last().unwrap().as_slice();
                        let indice = mesh_object.faces.iter().map(|v| [v.0 as u32, v.1 as u32, v.2 as u32]).collect::<Vec<[u32;3]>>().as_slice().concat();
                        indices.push(indice);
                        let ind = indices.last().unwrap().as_slice();
                        (vao, vbo, elem_buffer) = DriverValues::create_buffer_vec_norm(&mut driver, verts, ind);
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
                        let temp = buffer.iter().map(|f| {
                            let v = f.0.to_buffer();
                            vec![
                                v[0], 
                                v[1], 
                                v[2], 
                                f.1.0, 
                                f.1.1
                                ]
                        }).collect::<Vec<Vec<f32>>>();
                        let vertice = temp.iter().flatten().map(|v| v.clone()).collect::<Vec<f32>>();
                        vertices.push(vertice);
                        let verts = vertices.last().unwrap().as_slice();
                        let indice = mesh_object.faces.iter().map(|v| [v.0 as u32, v.1 as u32, v.2 as u32]).collect::<Vec<[u32;3]>>().as_slice().concat();
                        indices.push(indice);
                        let ind = indices.last().unwrap().as_slice();
                        (vao, vbo, elem_buffer) = DriverValues::create_buffer_vec_tex(&mut driver, verts, ind);
                    }
                    else {
                        let buffer = mesh_object.verts.clone();
                        let temp = buffer.iter().map(|f| {
                            let v = f.to_buffer();
                            vec![
                                v[0], 
                                v[1], 
                                v[2],
                                ]
                        }).collect::<Vec<Vec<f32>>>();
                        let vertice = temp.iter().flatten().map(|v| v.clone()).collect::<Vec<f32>>();
                        vertices.push(vertice);
                        let verts = vertices.last().unwrap().as_slice();
                        let indice = mesh_object.faces.iter().map(|v| [v.0 as u32, v.1 as u32, v.2 as u32]).collect::<Vec<[u32;3]>>().as_slice().concat();
                        indices.push(indice);
                        let ind = indices.last().unwrap().as_slice();
                        (vao, vbo, elem_buffer) = DriverValues::create_buffer_vec(&mut driver, verts, ind);
                    }
                }

                let shader_name = mesh_object.material.shader.shader_name.clone();

                let sh = DriverValues::register_shader(&mut driver, shader_name, mesh_object.material.shader.clone());
                let (name, vertex_shader, fragment_shader) = &driver.shader_stages[sh];

                let gl = driver.gl.as_ref().unwrap();

                shader_program = gl.CreateProgram();
                gl.AttachShader(shader_program, *vertex_shader);
                gl.AttachShader(shader_program, *fragment_shader);
                gl.LinkProgram(shader_program);
                vao_vec.push(vao);
                vbo_vec.push(vbo);
                elem_buff_vec.push(elem_buffer);
                shader_progs.push(shader_program);
            }
        }
        drop(mesh);
        Self { 
            p_mesh: p_mesh.clone(), 
            vao: Arc::new(Mutex::new(vao_vec)), 
            vbo: Arc::new(Mutex::new(vbo_vec)), 
            elem_buffer: Arc::new(Mutex::new(elem_buff_vec)), 
            shader_programs: Arc::new(Mutex::new(shader_progs)),
            vertices: Arc::new(Mutex::new(vertices)),
            indices: Arc::new(Mutex::new(indices)),
        }
    }
}

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
    pub shader_stages: Vec<(String, u32, u32)>,
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
                let mut render_line = Vec::<(u32, u32, u32, u32, Matrix34, u32, u32, HashMap<std::string::String, (Box<materials::ShaderType>, ShaderDataHint)>, usize)>::new();
                if camera.render_texture.is_some() {
                    driver.gl.as_ref().unwrap().BindFramebuffer(GL_FRAMEBUFFER, camera.render_texture.as_ref().unwrap().inner);
                    driver.gl.as_ref().unwrap().Viewport(0,0, camera.render_texture.as_ref().unwrap().width, camera.render_texture.as_ref().unwrap().height);

                }
                else{
                    driver.gl.as_ref().unwrap().Viewport(0, 0, GAME.window_x as i32, GAME.window_y as i32);
                }
                // println!("{}", pipeline.meshs.len());
                
                
                    let gl = driver.gl.as_ref().unwrap();
                    gl.Enable(GL_DEPTH_TEST);
                    
                    gl.DepthMask(GL_FALSE.0 as u8);
                    gl.DepthFunc(GL_ALWAYS);
                    // add other bufferes
                    // just for now we are going to add a vert output and frag output buffer
                    
                    for mesh in &mut pipeline.meshs.to_vec(){
                        mesh.update_mesh(driver, &camera);
                        let gl = driver.gl.as_ref().unwrap();
                        mesh.draw(gl, &camera);
                    }
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
}

pub struct PipelineValues {

}