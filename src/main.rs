
use std::{mem::{size_of_val, size_of}, str::SplitWhitespace, ffi::{CString, OsString}, any::TypeId, fs};
use bytemuck::try_cast_ref;
#[cfg(feature = "opengl")] use ogl33::*;
pub mod common;
use common::{vertex::*, filesystem::files::{FileSys, Reader}, *, materials::ShaderDescriptor, engine::gamesys::{Base, BaseToAny, GAME}, textures::Texture, matrices::*};


type Vertex = [f32; 3];

const VETRICES: [Vertex; 3] = [[-0.5, -0.5, 0.0], [0.5, -0.5, 0.0], [0.0, 0.5, 0.0]];

const VERT_SHADER: &str = r#"#version 330 core
layout (location = 0) in vec3 pos;
void main() {
    gl_Position = vec4(pos.x, pos.y, pos.z, 1.0);
}"#;

const FRAG_SHADER: &str = r#"#version 330 core
out vec4 final_color;
void main(){
    final_color = vec4(1.0,0.4,0.2,1.0);
}
"#;

fn generate_verts() -> Vec<Vec3>{
    let mut verts = Vec::<Vec3>::new();
    verts.push(Vec3::new(-0.5, 0.5, 0.0));
    verts.push(Vec3::new(-0.5, -0.5, 0.0));
    verts.push(Vec3::new(0.0, 0.5, 0.0));

    return verts;
}
#[cfg(feature = "opengl")]
fn generate_vertices(verts: &Vec<Vec3>, indices: &Vec<i32>){
    unsafe {
        let mut a = verts.iter().map(|x| x.to_buffer()).collect::<Vec<_>>();
        let vertices = a.as_mut_slice();

        let mut indexes = indices.as_slice();

        let mut vao = 0;
        glGenVertexArrays(1, &mut vao);
        assert_ne!(vao, 0);

        glBindVertexArray(vao);
        
        let mut vbo = 0;
        glGenBuffers(1, &mut vbo);
        assert_ne!(vbo, 0);

        glBindBuffer(GL_ARRAY_BUFFER, vbo);

        glBufferData(GL_ARRAY_BUFFER, (size_of_val(&vertices)) as isize, vertices.as_ptr().cast(), GL_STATIC_DRAW);
        glVertexAttribPointer(0, 3, GL_FLOAT, GL_FALSE, size_of::<Vertex>() as i32, 0 as *const _,);

        let mut elem_buf = 0;
        glGenBuffers(1, &mut elem_buf);
        assert_ne!(elem_buf, 0);

        glBindBuffer(GL_ELEMENT_ARRAY_BUFFER, elem_buf);

        glBufferData(GL_ELEMENT_ARRAY_BUFFER, (size_of_val(&indexes)) as isize, indexes.as_ptr().cast(), GL_STATIC_DRAW);
        

        glEnableVertexAttribArray(0);
    }
}

fn compare(input: &str) -> bool {
    return input.ends_with(".glsl") || input.ends_with(".vert") || input.ends_with(".frag") || input.ends_with(".comp");
}

fn include_shaders() -> glsl_include::Context<'static> {
    let path: String = APP_DIR.clone().to_owned() + "\\assets\\shaders\\";
    let directory = fs::read_dir(path).unwrap();
    let mut context: glsl_include::Context = glsl_include::Context::new();
    for mut path in directory {
        let mut path_unwraped = path.unwrap();
        let path_path = path_unwraped.path();
        if(path_path.is_file() && compare(path_unwraped.file_name().to_str().unwrap())){
            let mut path_file = path_unwraped.file_name();
            let mut path_file_str = path_file.as_os_str().to_str().unwrap();
            
            let mut path_data = path_path.display();
            let mut path_data_string = path_data.to_string();
            let mut path_data_str = path_data_string.as_str();
            context.include(path_file_str, path_data_str);
        }
    
    }
    return context;
}
#[cfg(feature = "opengl")]
fn generate_shader(frag: &str, vert: &str) -> GLuint {
    unsafe{
        let context = include_shaders();
        let vert1 = context.expand(vert).unwrap();
        let frag1 = context.expand(frag).unwrap();
        let vertex_shader = glCreateShader(GL_VERTEX_SHADER);
        assert_ne!(vertex_shader, 0);
        println!("3 {}", glGetError());
        glShaderSource(
            vertex_shader,
            1,
            &(vert1.as_bytes().as_ptr().cast()),
            &(vert1.len().try_into().unwrap()),
        );
        
        glCompileShader(vertex_shader);
        println!("4 {}", glGetError());
        let mut success = 0;
        glGetShaderiv(vertex_shader, GL_COMPILE_STATUS, &mut success);

        if(success == 0){
            let mut v: Vec<u8> = Vec::with_capacity(1024);
            let mut log_len = 0_i32;
            glGetShaderInfoLog(
                vertex_shader,
                1024,
                &mut log_len,
                v.as_mut_ptr().cast()
            );

            v.set_len(log_len.try_into().unwrap());
            panic!("Vertex Compile Error: {}", String::from_utf8_lossy(&v));

        }

        let fragment_shader = glCreateShader(GL_FRAGMENT_SHADER);
        assert_ne!(fragment_shader, 0);

        glShaderSource(
            fragment_shader,
            1,
            &(frag1.as_bytes().as_ptr().cast()),
            &(frag1.len().try_into().unwrap()),
        );
        glCompileShader(fragment_shader);
        println!("5 {}", glGetError());
        let mut success = 0;
        glGetShaderiv(fragment_shader, GL_COMPILE_STATUS, &mut success);
        if(success == 0){
            let mut v: Vec<u8> = Vec::with_capacity(1024);
            let mut log_len = 0_i32;
            glGetShaderInfoLog(fragment_shader, 1024, &mut log_len, v.as_mut_ptr().cast());
            v.set_len(log_len.try_into().unwrap());
            panic!("Fragment Compile Error: {}", String::from_utf8_lossy(&v));
        }
        
        let shader_program = glCreateProgram();
        glAttachShader(shader_program, vertex_shader);
        glAttachShader(shader_program, fragment_shader);
        glLinkProgram(shader_program);
        println!("6 {}", glGetError());
        let mut success = 0;
        glGetProgramiv(shader_program, GL_LINK_STATUS, &mut success);
        if success == 0 {
            let mut v: Vec<u8> = Vec::with_capacity(1024);
            let mut log_len = 0_i32;
            glGetProgramInfoLog(
                shader_program,
                1024,
                &mut log_len,
                v.as_mut_ptr().cast(),
            );
            v.set_len(log_len.try_into().unwrap());
            panic!("Program Link Error: {}", String::from_utf8_lossy(&v));
        }
        glUseProgram(shader_program);
        glDeleteShader(vertex_shader);
        glDeleteShader(fragment_shader);
        return shader_program;
    }
}
#[cfg(feature = "opengl")]
fn add_shader_params(program: GLuint, description: Box<&dyn ShaderDescriptor>){
    unsafe{
        let count = description.get_num_values();

        for i in 0..count {
            let type_id = description.get_value_type(i);
            let param = description.get_value(i).b;
            let name = CString::new(description.get_value_name(i)).unwrap();
            let size = size_of_val(&description.get_value(i));
            
            let paramLocation = glGetUniformLocation(program, name.as_ptr());
            let vec3_type = TypeId::of::<Vec3>();
            if(type_id == TypeId::of::<Vec3>()){
                glUniform3fv(paramLocation, size as i32, param.as_ref().as_any().downcast_ref::<Vec3>().unwrap().to_buffer().as_ptr());
            }
            if(type_id == TypeId::of::<Vec4>()){
                glUniform4fv(paramLocation, size as i32, param.as_ref().as_any().downcast_ref::<Vec4>().unwrap().to_buffer().as_ptr());
            }
            if(type_id == TypeId::of::<Vec<Vec3>>()){
                let mut vparam = param.as_ref().as_any().downcast_ref::<Vec<Vec3>>().unwrap().iter().map(|x| x.to_buffer()).collect::<Vec<_>>();
                let mut vvparam: Vec<f32> = vec![];
                for vec in vparam {
                    vvparam.push(vec[0]);
                    vvparam.push(vec[1]);
                    vvparam.push(vec[2]);
                }
                glUniform3fv(paramLocation, size as i32, vvparam.as_ptr())
            }
            if(type_id == TypeId::of::<Vec<Vec4>>()){
                let mut vparam = param.as_ref().as_any().downcast_ref::<Vec<Vec4>>().unwrap().iter().map(|x| x.to_buffer()).collect::<Vec<_>>();
                let mut vvparam: Vec<f32> = vec![];
                for vec in vparam {
                    vvparam.push(vec[0]);
                    vvparam.push(vec[1]);
                    vvparam.push(vec[2]);
                    vvparam.push(vec[3]);
                }
                glUniform3fv(paramLocation, size as i32, vvparam.as_ptr())
            }
            if(type_id == TypeId::of::<Matrix33>()){
                glUniformMatrix3fv(paramLocation, size as i32, GL_TRUE, param.as_ref().as_any().downcast_ref::<Matrix33>().unwrap().to_buffer().as_ptr())
            }
            if(type_id == TypeId::of::<Matrix34>()){
                glUniformMatrix3x4fv(paramLocation, size as i32, GL_TRUE, param.as_ref().as_any().downcast_ref::<Matrix34>().unwrap().to_buffer().as_ptr());
            }
        }
    }
}
#[cfg(feature = "opengl")]
fn oldMain() {
    let mut vec: Vec3 = Vec3::new(1,1,1);
    let mut vec2: Vec3 = Vec3::new(0,0,1);
    vec = vec.cross(vec2);
    
    let mut f: FileSys = FileSys::new();
    f.open("ASSETS:models\\fourteenth_sonic.obj");
    println!("{}", f.checkFileExt());
    println!("{}", vec);
    
    //let mut obj: MeshFile = MeshFile::new();
    
    let sdl = SDL::init(InitFlags::Everything).expect("couldn't start SDL");
    sdl.gl_set_attribute(SdlGlAttr::MajorVersion, 3).unwrap();
    sdl.gl_set_attribute(SdlGlAttr::MinorVersion, 3).unwrap();
    sdl.gl_set_attribute(SdlGlAttr::Profile, GlProfile::Core).unwrap();
    

    let _win = sdl
        .create_gl_window(
            "Hello Window",
            WindowPosition::Centered,
            800,
            600,
            WindowFlags::Shown,
        ).expect("couldn't make a window and context");
    unsafe{
        
        load_gl_with(|f_name| _win.get_proc_address(f_name));
        glClearColor(0.2, 0.3, 0.3, 1.0);
        glViewport(0, 0, 800, 600);
        let mut vao = 0;
        glGenVertexArrays(1, &mut vao);
        assert_ne!(vao, 0);

        glBindVertexArray(vao);
        
        let mut vbo = 0;
        glGenBuffers(1, &mut vbo);
        assert_ne!(vbo, 0);

        glBindBuffer(GL_ARRAY_BUFFER, vbo);

 
        println!("{}", size_of_val(&VETRICES));
        glBufferData(GL_ARRAY_BUFFER, (size_of_val(&VETRICES)) as isize, VETRICES.as_ptr().cast(), GL_STATIC_DRAW);
        glVertexAttribPointer(0, 3, GL_FLOAT, GL_FALSE, size_of::<Vertex>() as i32, 0 as *const _,);
        glEnableVertexAttribArray(0);

        //Vertex Shader!
        let vertex_shader = glCreateShader(GL_VERTEX_SHADER);
        assert_ne!(vertex_shader, 0);
        println!("3 {}", glGetError());
        glShaderSource(
            vertex_shader,
            1,
            &(VERT_SHADER.as_bytes().as_ptr().cast()),
            &(VERT_SHADER.len().try_into().unwrap()),
        );
        
        glCompileShader(vertex_shader);
        println!("4 {}", glGetError());
        let mut success = 0;
        glGetShaderiv(vertex_shader, GL_COMPILE_STATUS, &mut success);

        if(success == 0){
            let mut v: Vec<u8> = Vec::with_capacity(1024);
            let mut log_len = 0_i32;
            glGetShaderInfoLog(
                vertex_shader,
                1024,
                &mut log_len,
                v.as_mut_ptr().cast()
            );

            v.set_len(log_len.try_into().unwrap());
            panic!("Vertex Compile Error: {}", String::from_utf8_lossy(&v));

        }

        let fragment_shader = glCreateShader(GL_FRAGMENT_SHADER);
        assert_ne!(fragment_shader, 0);

        glShaderSource(
            fragment_shader,
            1,
            &(FRAG_SHADER.as_bytes().as_ptr().cast()),
            &(FRAG_SHADER.len().try_into().unwrap()),
        );
        glCompileShader(fragment_shader);
        println!("5 {}", glGetError());
        let mut success = 0;
        glGetShaderiv(fragment_shader, GL_COMPILE_STATUS, &mut success);
        if(success == 0){
            let mut v: Vec<u8> = Vec::with_capacity(1024);
            let mut log_len = 0_i32;
            glGetShaderInfoLog(fragment_shader, 1024, &mut log_len, v.as_mut_ptr().cast());
            v.set_len(log_len.try_into().unwrap());
            panic!("Fragment Compile Error: {}", String::from_utf8_lossy(&v));
        }
        
        let shader_program = glCreateProgram();
        glAttachShader(shader_program, vertex_shader);
        glAttachShader(shader_program, fragment_shader);
        glLinkProgram(shader_program);
        println!("6 {}", glGetError());
        let mut success = 0;
        glGetProgramiv(shader_program, GL_LINK_STATUS, &mut success);
        if success == 0 {
            let mut v: Vec<u8> = Vec::with_capacity(1024);
            let mut log_len = 0_i32;
            glGetProgramInfoLog(
                shader_program,
                1024,
                &mut log_len,
                v.as_mut_ptr().cast(),
            );
            v.set_len(log_len.try_into().unwrap());
            panic!("Program Link Error: {}", String::from_utf8_lossy(&v));
        }
        glUseProgram(shader_program);
        glDeleteShader(vertex_shader);
        glDeleteShader(fragment_shader);
        println!("7 {}", glGetError());
        
        _win.set_swap_interval(SwapInterval::Vsync);
        glDisable(GL_CULL_FACE);
    }
    'main_loop: loop {
        while let Some(event) = sdl.poll_events().and_then(Result::ok){
            match event{
                Event::Quit(_) => break 'main_loop, _ => (),
            }

        } 
        unsafe{
            glClear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT);
            glDrawArrays(GL_TRIANGLES, 0, 3);
            
        }
        _win.swap_window();
    }
}

fn main(){
    unsafe{
    GAME.init();
    }
}