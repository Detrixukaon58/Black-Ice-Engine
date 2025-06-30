use crate::black_ice::{self, common::engine::{asset_mgr::{self, AssetManager}, pipeline::RenderPipelineSystem}};
use parking_lot::Mutex;
use std::{any::TypeId, collections::HashMap, path::PathBuf, sync::Arc};
use crate::black_ice::common::{Env, Ptr, Base};
use shaderc::ShaderKind;


use super::AssetResource;

#[derive(Clone, PartialEq)]
pub struct StructDescriptor {
    pub name: String,
    pub members: Vec<VariableDescriptorEnum>,
    pub binding: u32,
}
#[derive(Clone, PartialEq)]
pub struct VariableDescriptor {
    pub name: String,
    pub offset: u32,
    pub binding: u32,

}

#[derive(Clone, PartialEq)]
pub enum VariableDescriptorEnum {
    Variable (VariableDescriptor),
    Struct (StructDescriptor),
    NoVar
}
#[derive(Clone, Default)]
pub struct ShaderStageDescriptor {
    pub data: Vec<VariableDescriptorEnum>
}


impl ShaderStageDescriptor {

    pub fn get(&self, name: String) -> VariableDescriptorEnum {
        let name_split = name.split('.');
        let mut var: VariableDescriptorEnum = VariableDescriptorEnum::NoVar;
        for split in name_split {
            var = match &mut var {
                VariableDescriptorEnum::NoVar=> {
                    // we need to select the first var we want
                    let dat = self.data.clone();
                    let val = dat.into_iter().find(|x| {
                        match x {
                            VariableDescriptorEnum::Variable(variable_descriptor) => variable_descriptor.name == split,
                            VariableDescriptorEnum::Struct(struct_descriptor) => struct_descriptor.name == split,
                            VariableDescriptorEnum::NoVar => false,
                        }
                    }).unwrap_or(VariableDescriptorEnum::NoVar).clone();
                    if val == VariableDescriptorEnum::NoVar {
                        return VariableDescriptorEnum::NoVar;
                    }
                    else{
                        val
                    }
                },
                VariableDescriptorEnum::Struct(struct_descriptor) => {
                    let val = struct_descriptor.members.clone().into_iter().find(|x| {
                        match x {
                            VariableDescriptorEnum::Variable(variable_descriptor) => variable_descriptor.name == split,
                            VariableDescriptorEnum::Struct(struct_descriptor) => struct_descriptor.name == split,
                            VariableDescriptorEnum::NoVar => false,
                        }
                    }).unwrap_or(VariableDescriptorEnum::NoVar).clone();
                    if val == VariableDescriptorEnum::NoVar {
                        return VariableDescriptorEnum::NoVar;
                    }
                    else{
                        val
                    }
                },
                VariableDescriptorEnum::Variable(variable_descriptor) => {
                    if variable_descriptor.name == split {
                        break;
                    }
                    else{
                        return VariableDescriptorEnum::NoVar;
                    }
                }
            };
        }
        return var;
    }

    pub fn get_binding(&self, name: String) -> i32 {
        let value = self.get(name);
        match value {
            VariableDescriptorEnum::Variable(variable_descriptor) => variable_descriptor.binding as i32,
            VariableDescriptorEnum::Struct(struct_descriptor) => struct_descriptor.binding as i32,
            VariableDescriptorEnum::NoVar => -1,
        }
    }

    pub fn append(&mut self, other: &mut Self) {
        self.data.append(&mut other.data);
    }

    pub fn insert(&mut self, var: VariableDescriptorEnum) {
        self.data.push(var);
    }

    pub fn insert_variable(&mut self, var_name: String, var_offset: u32, var_binding: u32) {
        self.data.push(
            VariableDescriptorEnum::Variable(
                VariableDescriptor { name: var_name, offset: var_offset, binding: var_binding }
            )
        );
    }

    // fn recurse_variable(block: ReflectBlockVariable) -> VariableDescriptorEnum {
    //     match block.type_description {
    //         Some(t) => {
    //             match t {

    //             }
    //         },
    //         None => {

    //         }
    //     }
    // }

    pub fn new() -> Self {
        Self { data: Vec::new() }
    }

}

#[derive(Clone)]
pub enum ShaderDataType {
    Integer(i32),
    Boolean(bool),
    UnsignedInteger(u32),
    Float(f32),
    Double(f64),
    Vec3([f32; 3]),
    Vec4([f32; 4]),
    Vec2([f32; 2]),
    IVec3([i32; 3]),
    IVec4([i32; 4]),
    IVec2([i32; 2]),
    UVec3([u32; 3]),
    UVec4([u32; 4]),
    UVec2([u32; 2]),
    DVec3([f64; 3]),
    DVec4([f64; 4]),
    DVec2([f64; 2]),
    Sampler2D(Vec<u32>, u32, u32),

}

#[derive(Clone)]
pub enum ImageFormat {
    Unknown ,
    Rgba32f ,
    Rgba16f ,
    R32f ,
    Rgba8 ,
    Rgba8Snorm ,
    Rg32f ,
    Rg16f ,
    R11fG11fB10f ,
    R16f ,
    Rgba16 ,
    Rgb10A2 ,
    Rg16 ,
    Rg8 ,
    R16 ,
    R8 ,
    Rgba16Snorm ,
    Rg16Snorm ,
    Rg8Snorm ,
    R16Snorm ,
    R8Snorm ,
    Rgba32i ,
    Rgba16i ,
    Rgba8i ,
    R32i ,
    Rg32i ,
    Rg16i ,
    Rg8i ,
    R16i ,
    R8i ,
    Rgba32ui ,
    Rgba16ui ,
    Rgba8ui ,
    R32ui ,
    Rgb10a2ui ,
    Rg32ui ,
    Rg16ui ,
    Rg8ui ,
    R16ui ,
    R8ui ,
    R64ui ,
    R64i ,
}

#[derive(Clone)]
pub enum DataType {
    I64,
    I32,
    I16,
    I8,
    U64,
    U32,
    U16,
    U8,
    Double,
    F32,
    F16,
    F8,
    Boolean,
    IVec4 { bits: u32},
    IVec3 { bits: u32},
    IVec2 { bits: u32},
    UVec4 { bits: u32},
    UVec3 { bits: u32},
    UVec2 { bits: u32},
    DVec4,// these aren't used, but must be kept for future proofing
    DVec3,
    DVec2,
    Vec4 { bits: u32},
    Vec3 { bits: u32},
    Vec2 { bits: u32},
    BVec3,
    BVec4,
    BVec2,
    FMatrix4 { bits: u32,  row_major: bool,  stride: usize},
    FMatrix3 { bits: u32,  row_major: bool,  stride: usize},
    FMatrix2 { bits: u32,  row_major: bool,  stride: usize},
    IMatrix4 { bits: u32,  row_major: bool,  stride: usize},
    IMatrix3 { bits: u32,  row_major: bool,  stride: usize},
    IMatrix2 { bits: u32,  row_major: bool,  stride: usize},
    UMatrix4 { bits: u32,  row_major: bool,  stride: usize},
    UMatrix3 { bits: u32,  row_major: bool,  stride: usize},
    UMatrix2 { bits: u32,  row_major: bool,  stride: usize},
    BMatrix4 { bits: u32,  row_major: bool,  stride: usize},
    BMatrix3 { bits: u32,  row_major: bool,  stride: usize},
    BMatrix2 { bits: u32,  row_major: bool,  stride: usize},
    Sampler1D { depth: u32,  bit_depth: u32,  is_signed: bool,  is_float: bool,  format: ImageFormat},
    Sampler2D { depth: u32,  bit_depth: u32,  is_signed: bool,  is_float: bool,  format: ImageFormat},
    Sampler3D { depth: u32,  bit_depth: u32,  is_signed: bool,  is_float: bool,  format: ImageFormat},
    SamplerCube { depth: u32,  bit_depth: u32,  is_signed: bool,  is_float: bool,  format: ImageFormat},
    SamplerRect { depth: u32,  bit_depth: u32,  is_signed: bool,  is_float: bool,  format: ImageFormat},
    Void
}

#[derive(Clone)]
pub enum ShaderDataHint {
    Uniform,
    In,
    Out,
    InOut,
    Buffer {layout:u32},
}

pub type ShaderPtr = usize;


#[derive(Clone)]
pub struct Shader {
    pub shader_stages: Vec<ShaderPtr>,
    pub shader_name: String,
    pub asset_path: String,
}
struct ShaderToken {
    pub shader_name: String,
    pub shader_type: ShaderType,
    pub shader_lang: ShaderLang,
    pub shader_inout_datas: Vec<(String, DataType, ShaderDataHint)>,
    pub shader_code: Vec<u8>,
    pub is_compiled: bool,

}
pub fn compare(input: &str) -> bool {
    return input.ends_with(".glsl") || input.ends_with(".vert") || input.ends_with(".frag") || input.ends_with(".comp");
}

#[derive(Clone)]
pub enum ShaderType {
    Compute,
    Fragment,
    Vertex,
    Infer,
}

#[derive(Clone)]
pub struct ShaderData {
    pub data: Arc<Mutex<Vec<u8>>>,
    pub compiled_data: Option<Arc<Mutex<Vec<u8>>>>,
    pub descriptor: ShaderStageDescriptor,
}

impl ShaderData {
    fn include_shaders() -> glsl_include::Context<'static> {
        // load shaders that have been registered through the asset pack system
        // these usually will be tested for when we look at the file metadata!!
        let mut context: glsl_include::Context = glsl_include::Context::new();
            
        // lets get all glsl and hlsl files

        unsafe{

            let p_render_sys = Env::get_render_sys();
            let render_sys = p_render_sys.read();
            for (shader_name, (_asset_path, data)) in &render_sys.registered_shaders {
                // we will now need to load each shader that can be imported!!
                
                context.include(shader_name, &String::from_utf8(data.clone()).unwrap());
            }
        }

        

        // go through the directory and find all includable shaders!!!

        // Includable shader:
        // .glsl
        // .hlsl
        // 
        
        


        return context;
    }

    pub fn compile(&mut self, shader_type: ShaderType, _shader_lang: ShaderLang, name: String){
        let compiler = shaderc::Compiler::new().expect("Failed to init shaderc!!");
        let data_ptr = self.data.clone();
        let data = data_ptr.lock();
        let text = std::str::from_utf8(&data).expect("Data is not of proper UTF8 form!!");
        let context = ShaderData::include_shaders();
        let temp_text = context.expand(text).expect("Failed to include neseccary shaders!!");

        let mut options = shaderc::CompileOptions::new().expect("Failed to create shader options!!");
        options.set_auto_map_locations(true);
        options.set_auto_bind_uniforms(true);
        options.set_source_language(shaderc::SourceLanguage::GLSL);
        
        #[cfg(feature = "opengl")] options.set_target_env(shaderc::TargetEnv::OpenGL, shaderc::EnvVersion::OpenGL4_5 as u32);
        #[cfg(feature = "vulkan")] options.set_target_env(shaderc::TargetEnv::Vulkan, shaderc::EnvVersion::Vulkan1_0 as u32);
        
        let shader_kind = match shader_type {
            ShaderType::Compute => ShaderKind::Compute,
            ShaderType::Fragment => ShaderKind::Fragment,
            ShaderType::Vertex => ShaderKind::Vertex,
            ShaderType::Infer => ShaderKind::InferFromSource,
        };
        
        let artifact = compiler.compile_into_spirv(temp_text.as_str(), shader_kind, name.as_str(), "main", Some(&options));
        let temp = artifact.expect("Failed to compile shader!!!");
        

        // lets get the shader variable info!!
        // let module = ShaderModule::load_u8_data(d.as_slice()).expect("Failed to load spirv binary. implementation must be wrong.");
        // let uniforms = module.enumerate_descriptor_bindings(None).expect("Failed to get uniform bindings");
        // for uniform in uniforms {
        //     let _type = uniform.descriptor_type;

        // }
        let module = spirv_cross::spirv::Module::from_words(temp.as_binary());
        let ast = spirv_cross::spirv::Ast::<spirv_cross::glsl::Target>::parse(&module).unwrap();
        let resources = ast.get_shader_resources().unwrap();
        for uniform in resources.uniform_buffers {
            let name = uniform.name.clone();
            let id = uniform.id.clone();
            let binding = ast.get_decoration(id.clone(), spirv_cross::spirv::Decoration::Binding).unwrap();
            let _type = ast.get_type(uniform.base_type_id).unwrap_or(spirv_cross::spirv::Type::Void);
            match _type {
                spirv_cross::spirv::Type::Struct { member_types, array: _ } => {
                    let member_count = member_types.len() as u32;
                    let mut members: Vec<VariableDescriptorEnum> = vec![];
                    for index in 0..member_count {
                        let member_name = ast.get_member_name(uniform.base_type_id.clone(), index).unwrap();
                        let member_offset = ast.get_member_decoration(id.clone(), index, spirv_cross::spirv::Decoration::Offset).unwrap();
                        let member_binding = ast.get_member_decoration(id.clone(), index, spirv_cross::spirv::Decoration::Binding).unwrap();

                        members.push(VariableDescriptorEnum::Variable(VariableDescriptor { name: member_name, offset: member_offset, binding: member_binding }));
                    }
                    self.descriptor.insert(VariableDescriptorEnum::Struct(StructDescriptor { name: name, members: members, binding: binding }));
                }
                _ => {
                    let offset = ast.get_decoration(id.clone(), spirv_cross::spirv::Decoration::Offset).unwrap();

                    self.descriptor.insert_variable(name, offset, binding);
                }
            }

        }
        
        self.compiled_data = Some(Arc::new(Mutex::new(temp.as_binary_u8().to_vec())));
    }

    pub fn infer_shader_type(&mut self) -> ShaderType {
        let re = fancy_regex::Regex::new(r"(?<=#pragma shader_type\()\b[a-z]+\b(?=\))").unwrap();
        let mut data = self.data.lock();
        let text = std::str::from_utf8(data.as_slice()).expect("Failed to parse shader data!!");
        
        let mut capture = re.captures(text).expect("Failed to start regex!!").expect("Failed to get capture!!");
        let mut value = capture.get(0).expect("No shader type defined for shader!! Please add \"#pragma shader_type(shader type)\" to your file!!").as_str();
        
        match value {
            "vertex" => ShaderType::Vertex,
            "fragment" => ShaderType::Fragment,
            "compute" => ShaderType::Compute,
            _ => panic!("Shader type not defined!!")
        }

    }

    pub fn get_hlsl_shaders(&mut self) -> Vec<(String, ShaderType)> {
        let re = fancy_regex::Regex::new(r"(?<=#pragma )\b[a-z]+\b \b[a-z,A-Z]+\b").expect("Failed to init regex!!");
        let mut data = self.data.lock();
        let text = std::str::from_utf8(data.as_slice()).expect("Failed to parse shader data!!!");

        let mut result = vec![];

        for c in re.captures_iter(text) {
            let capture = c.expect("Failed to get capture");
            let value = capture.get(0).expect("HLSL shader has no shader functions defined!! use \"#pragma function_name shader_type\" to declare the shader functions!!");
            result.push(value.as_str());
        }

        let mut result2 = vec![];

        for v in result {
            let mut temp: Vec<&str> = v.split(" ").collect();
            let name = String::from(temp[1]);
            let mut shader_type = ShaderType::Compute;
            match temp[0]{
                "vertex" => shader_type = ShaderType::Vertex,
                "fragment" => shader_type = ShaderType::Fragment,
                "compute" => shader_type = ShaderType::Compute,
                _ => continue,
            }
            result2.push((name, shader_type));
        }

        result2
    }

    pub fn hlsl_compile(&mut self, name:String) {
        let shader_entries = self.get_hlsl_shaders();
        let compiler = shaderc::Compiler::new().expect("Failed to get compiler!!");
        let mut options = shaderc::CompileOptions::new().expect("Failed to load compiler options!!");
        options.set_source_language(shaderc::SourceLanguage::HLSL);
        options.set_hlsl_io_mapping(true);
        options.set_hlsl_offsets(true);
        #[cfg(feature = "opengl")] options.set_target_env(shaderc::TargetEnv::OpenGL, shaderc::EnvVersion::OpenGL4_5 as u32);
        #[cfg(feature = "vulkan")] options.set_target_env(shaderc::TargetEnv::Vulkan, shaderc::EnvVersion::Vulkan1_0 as u32);

        options.set_auto_bind_uniforms(true);
        options.set_auto_map_locations(true);

        let mut data = self.data.lock();
        
        let artifact = compiler.compile_into_spirv(std::str::from_utf8(data.as_slice()).expect("Failed to parse shader code!!"), ShaderKind::InferFromSource, &name, "main", Some(&options));
        let temp = artifact.expect("Failed to compile shader!!");

        self.compiled_data = Some(Arc::new(Mutex::new(temp.as_binary_u8().to_vec())));
    } 
}

#[derive(Copy, Clone)]
pub enum ShaderLang {
    Glsl,
    Hlsl,
    Pssl,
    GodotShader,
}

#[derive(Clone)]
pub struct ShaderStage {
    pub stage_name: String,
    pub shader_data: ShaderData,
    pub shader_type: ShaderType,
    pub shader_lang: ShaderLang,
    pub shader_inout: Vec<(String, DataType, ShaderDataHint)>,
    pub shader_id: Option<Vec<u32>>,
}
impl ShaderStage {
    fn new(shader_name: String, shader_type: ShaderType, shader_lang: ShaderLang, mut shader_data: ShaderData, shader_inout: Vec<(String, DataType, ShaderDataHint)>) -> ShaderStage {
        
        let shader_kind = match (shader_type) {
            ShaderType::Fragment => shaderc::ShaderKind::Fragment,
            ShaderType::Vertex => shaderc::ShaderKind::Vertex,
            ShaderType::Compute => shaderc::ShaderKind::Compute,
            ShaderType::Infer => shaderc::ShaderKind::InferFromSource
        };

        unsafe {
            
            ShaderStage {stage_name: shader_name, shader_data: shader_data, shader_type:shader_type, shader_lang:shader_lang,shader_inout:shader_inout, shader_id: None}
        }
    }

    fn read_uniforms(&mut self) -> HashMap<String, Box<&'static dyn Base>> {
        let hash: HashMap<String, Box<&'static dyn Base>> = HashMap::new();
        let file = &self.shader_data;
        
        return hash;
    }
}


impl Shader {
    
    fn parse_shad_file(data: &Vec<u8>) -> Vec<ShaderToken> {
        //tokenize the shader data
        #[derive(Clone)]
        pub enum Data {
            String(Vec<u8>),
            Vector(Vec<Vec<u8>>),
        }

        impl Data {
            pub fn as_str(&mut self) -> Vec<u8> {
                match self {
                    Data::String(s) => s.clone(),
                    _ => vec![]
                }
            }

            pub fn as_vec(&mut self) -> Vec<Vec<u8>> {
                match self {
                    Data::Vector(v) => v.clone(),
                    _ => vec![]
                }
            }
        }

        let mut tokens: Vec<ShaderToken> = vec![];

        let mut is_in_token: bool = false;
        let mut braket_count: u32 = 0;
        let mut is_quote: bool = false;
        let mut data_label: String = "".to_string();
        let mut d: Vec<u8> = vec![];
        let mut is_back: bool = false;
        let mut is_obtaining_data: bool = false;
        let mut data_list: HashMap<String,Data> = HashMap::new();
        let mut is_vector_data: bool = false;
        let mut vector_data: Vec<Vec<u8>> = vec![];
        let mut is_data: bool = false;
        let mut saved_data: Data = Data::String(vec![]);
        for c in data.as_slice() {
            match c {
                b'{' => {
                    if is_quote || is_vector_data{
                        d.push(c.clone());
                        continue;
                    }
                    if is_in_token {

                    }
                    else {
                        is_in_token = true;
                    }
                    braket_count += 1;
                },
                b'}' => {
                    if is_quote || is_vector_data{
                        d.push(c.clone());
                        continue;
                    }
                    if is_in_token && braket_count > 1{
                        
                    }
                    else if is_in_token && braket_count == 1 {
                        is_in_token = false;
                        // theoretically have reached the end of all!!
                        let mut name_default = Data::String(vec![b's',b'h',b'a',b'd',b'e',b'r']);
                        let mut name_temp = data_list.get("shader_name").unwrap_or(&name_default).clone().as_str();
                        let mut _shader_name = String::from_utf8(name_temp).expect("Failed to parse shader name!!");
                        let type_default = Data::String(b"compute".to_vec());
                        let _shader_type = match data_list.get("shader_type").unwrap_or(&type_default).clone().as_str().as_slice() {
                            b"fragment" => ShaderType::Fragment,
                            b"vertex" => ShaderType::Vertex,
                            _ => ShaderType::Compute,
                        };
                        let lang_default = Data::String(b"glsl".to_vec());
                        let mut _shader_lang = match data_list.get("shader_lang").unwrap_or(&lang_default).clone().as_str().as_slice() {
                            b"hlsl" => ShaderLang::Hlsl,
                            b"pgsl" => ShaderLang::Pssl,
                            b"godot" => ShaderLang::GodotShader,
                            _ => ShaderLang::Glsl,
                        };
                        let com_default = Data::String(b"false".to_vec());
                        let mut _is_compiled = match data_list.get("is_compiled").unwrap_or(&com_default).clone().as_str().as_slice() {
                            b"true" => true,
                            _ => false,
                        };
                        let mut inout_datas: Vec<(String, DataType, ShaderDataHint)> = vec![];
                        if data_list.contains_key("shader_inout_datas") {
                            let temp = data_list.get("shader_inout_datas").expect("No such value shader_inout_datas!!").clone().as_vec();
                            for val in temp 
                            {
                                unsafe {
                                    let string = String::from_utf8(val).expect("Failed to parse string!");
                                
                                    let temp_list: Vec<&str> = string.split(';').collect();
                                    
                                    let name = temp_list[0].to_string();
                                    let data_type = match temp_list[1] {
                                        "Integer" => DataType::I32,
                                        "Boolean" => DataType::Boolean,
                                        "UnssignedInteger" => DataType::U32,
                                        "Float" => DataType::F32,
                                        "Double" => DataType::Double,
                                        "Vec3" => DataType::Vec3 {bits: 32},
                                        "Vec4" => DataType::Vec4 {bits: 32},
                                        "Vec2" => DataType::Vec2 {bits: 32},
                                        "IVec3" => DataType::IVec3 {bits: 32},
                                        "IVec4" => DataType::IVec4 {bits: 32},
                                        "IVec2" => DataType::IVec2 {bits: 32},
                                        "UVec3" => DataType::UVec3 {bits: 32},
                                        "UVec4" => DataType::UVec4 {bits: 32},
                                        "UVec2" => DataType::UVec2 {bits: 32},
                                        "DVec3" => DataType::Vec3 {bits: 32},
                                        "DVec4" => DataType::Vec4 {bits: 32},
                                        "DVec2" => DataType::Vec2 {bits: 32},
                                        "Sampler2D" => DataType::Sampler2D { depth: 4, bit_depth: 32, is_signed: true, is_float: true, format: ImageFormat::Rgba32f },
                                        _ => DataType::F32,
                                    };
                                    let data_hint = match temp_list[2] {
                                        "Uniform" => ShaderDataHint::Uniform,
                                        "In" => ShaderDataHint::In,
                                        "Out" => ShaderDataHint::Out,
                                        "InOut" => ShaderDataHint::InOut,
                                        "Buffer" => ShaderDataHint::Buffer { layout: 0 },// we need to read the buffer layout
                                        _ => ShaderDataHint::Uniform,
                                    };

                                    inout_datas.push((name, data_type, data_hint));
                                    
                                }
                            }
                        }
                        let default_code = Data::String(b"".to_vec());

                        tokens.push(ShaderToken { 
                            shader_name: _shader_name, 
                            shader_type: _shader_type, 
                            shader_inout_datas: inout_datas, 
                            shader_code: data_list.get("shader_code").unwrap_or_else(|| {&default_code}).clone().as_str().clone(),
                            shader_lang: _shader_lang,
                            is_compiled: _is_compiled
                        });
                                    
                                
                            
                        
                        data_list.clear();
                    }

                    if braket_count >= 1 {
                        braket_count -= 1;
                    }
                },
                b'\"' => {
                    if is_back {
                        d.push(c.clone());
                        is_back = false;
                        continue;
                    }
                    if is_quote {
                        is_quote = false;
                        //check if the string is name for property
                        if is_data && !is_vector_data {
                            // this must be data
                            saved_data = Data::String(d.clone());
                            d.clear();
                        }
                        
                    }
                    else{
                        is_quote = true;
                    }
                },
                b'\\' => {
                    if is_back {
                        d.push(c.clone());
                        is_back = false;
                        continue;
                    }
                    else{
                        is_back = true;
                    }
                },
                b':' => {
                    if is_quote{
                        d.push(c.clone());
                        continue;
                    }
                    if !is_data {
                        is_data = true;
                        unsafe {
                            data_label = String::from_utf8(d.clone()).expect("Failed to parse String!!");
                            d.clear();
                        }
                    }
                },
                b',' => {
                    if is_quote{
                        d.push(c.clone());
                        continue;
                    }
                    if is_vector_data {
                        vector_data.push(d.clone());
                        d.clear();
                        continue;
                    }
                    if is_data{
                        is_data = false;
                        data_list.insert(data_label.clone(), saved_data.clone());
                        data_label.clear();
                        saved_data = Data::String(b"".to_vec());
                        continue;
                    }
                },
                b'[' => {
                    if is_quote {
                        d.push(c.clone());
                        continue;
                    }
                    if !is_vector_data && is_data{
                        is_vector_data = true;
                    }
                },
                b']' => {
                    if is_quote {
                        d.push(c.clone());
                        continue;
                    }
                    if is_vector_data && is_data{
                        is_vector_data = false;
                        // Now we submit the data in vec_datato data list
                        saved_data = Data::Vector(vector_data.clone());
                        vector_data.clear();
                    }
                },
                _ => {
                    if is_quote {
                        d.push(c.clone());
                    }
                }

            }
        }

        return tokens;
    }
}


impl AssetResource for Shader {
    fn new() -> Self {
        Self { shader_stages: vec![], shader_name: "".to_string(), asset_path: "".to_string() }

    }

    fn init(&mut self, data: std::sync::Arc<asset_mgr::AssetData>) {

        // we should check if the shader has already been registered!!

        if data.metadata["type"] != "Shader"{
            panic!("File is not a shader type!!");
        }
        else{
            // parse data and organise
            let mut shader_data = Vec::<u8>::new();
            unsafe {
            let p_render_sys = Env::get_render_sys();
            let mut render_sys = p_render_sys.write();
            let (a, b) = render_sys.registered_shaders[&data.asset_name].clone();
                shader_data = b;
            }
            let mut stages: Vec<ShaderStage> = vec![];
            let ext = data.metadata["ext"].clone();
            let path = PathBuf::from(data.asset_path.clone());
            let file_stem = String::from(path.file_name().unwrap().to_str().unwrap());
            let file_name = String::from(&file_stem[..file_stem.find(".").unwrap()]);
            unsafe {
                match ext.as_str() {
                    "shad" => {
                        // parse file and load shader stages
                        let mut tokens = Self::parse_shad_file(&shader_data);
                        for token in tokens {
                            let mut stage_ext = ".shad";
                            match token.shader_lang {
                                ShaderLang::Glsl => {
                                    match token.shader_type {
                                        ShaderType::Compute => stage_ext = ".comp",
                                        ShaderType::Fragment => stage_ext = ".frag",
                                        ShaderType::Vertex => stage_ext = ".vert",
                                        ShaderType::Infer => stage_ext = ".glsl"
                                    }
                                },
                                ShaderLang::Hlsl => stage_ext = ".hlsl",
                                ShaderLang::GodotShader => stage_ext = ".gdshad",
                                ShaderLang::Pssl => stage_ext = ".pfx",
                            }
                            if !token.is_compiled {
                                let mut shader_data: ShaderData = ShaderData { data: Arc::new(Mutex::new(token.shader_code.clone())), compiled_data: None , descriptor: ShaderStageDescriptor::default()};
                                shader_data.compile(token.shader_type.clone(), token.shader_lang.clone(), file_name.clone() + stage_ext);
                                stages.push(ShaderStage::new(file_name.clone() + stage_ext, token.shader_type, token.shader_lang.clone(), shader_data, token.shader_inout_datas));
                            }
                            else {
                                let code = token.shader_code.clone();
                                let _code_u32 = code.align_to::<u32>().1;
                                let shader_data: ShaderData = ShaderData { data: Arc::new(Mutex::new(token.shader_code.clone())), compiled_data: Some(Arc::new(Mutex::new(code.to_vec()))), descriptor: ShaderStageDescriptor::default() };
                                stages.push(ShaderStage::new(file_name.clone() + stage_ext, token.shader_type, token.shader_lang.clone(), shader_data, token.shader_inout_datas));
                            }
                        }
                    },
                    "vert" => {
                        //parse as single stage
                        let mut vec = shader_data.clone();
                        let mut shader_data: ShaderData = ShaderData { data: Arc::new(Mutex::new(vec)), compiled_data: None, descriptor: ShaderStageDescriptor::default() };
                        shader_data.compile(ShaderType::Vertex, ShaderLang::Glsl, file_stem.clone());
                        stages.push(ShaderStage::new(file_stem.clone(), ShaderType::Vertex, ShaderLang::Glsl, shader_data, vec![]));
                    },
                    "frag" => {
                        //parse as single stage
                        let mut vec = &shader_data;
                        let mut shader_data: ShaderData = ShaderData { data: Arc::new(Mutex::new(vec.clone())), compiled_data: None, descriptor: ShaderStageDescriptor::default() };
                        shader_data.compile(ShaderType::Fragment, ShaderLang::Glsl, file_stem.clone());
                        stages.push(ShaderStage::new(file_stem.clone(), ShaderType::Fragment, ShaderLang::Glsl, shader_data, vec![]));
                    },
                    "glsl" => {
                        //parse as single stage
                        let mut vec = &shader_data;
                        let mut shader_data: ShaderData = ShaderData { data: Arc::new(Mutex::new(vec.clone())), compiled_data: None, descriptor: ShaderStageDescriptor::default() };
                        shader_data.compile(ShaderType::Infer, ShaderLang::Glsl, file_stem.clone());
                        stages.push(ShaderStage::new(file_stem.clone(), ShaderType::Infer, ShaderLang::Glsl, shader_data, vec![]));
                    },
                    "comp" => {
                        //parse as single stage
                        let mut vec = &shader_data;
                        let mut shader_data: ShaderData = ShaderData { data: Arc::new(Mutex::new(vec.clone())), compiled_data: None, descriptor: ShaderStageDescriptor::default() };
                        shader_data.compile(ShaderType::Compute, ShaderLang::Glsl, file_stem.clone());
                        stages.push(ShaderStage::new(file_stem.clone(), ShaderType::Compute, ShaderLang::Glsl, shader_data, vec![]));
                    },
                    "hlsl" => {
                        //parse as single stage
                        let mut vec = &shader_data;
                        let mut shader_data: ShaderData = ShaderData { data: Arc::new(Mutex::new(vec.clone())), compiled_data: None, descriptor: ShaderStageDescriptor::default() };
                        shader_data.hlsl_compile(file_stem.clone());
                        stages.push(ShaderStage::new(file_stem.clone(), ShaderType::Infer, ShaderLang::Hlsl, shader_data, vec![]));
                    },
                    "fx" => {
                        //parse as single stage
                        let mut vec = &shader_data;
                        let mut shader_data: ShaderData = ShaderData { data: Arc::new(Mutex::new(vec.clone())), compiled_data: None, descriptor: ShaderStageDescriptor::default() };
                        shader_data.hlsl_compile(file_stem.clone());
                        stages.push(ShaderStage::new(file_stem.clone(), ShaderType::Infer, ShaderLang::Hlsl, shader_data, vec![]));
                    },
                    "pfx" => {
                        //parse as single stage
                        let mut vec = &shader_data;
                        let mut shader_data: ShaderData = ShaderData { data: Arc::new(Mutex::new(vec.clone())), compiled_data: None, descriptor: ShaderStageDescriptor::default() };
                        stages.push(ShaderStage::new(file_stem.clone(), ShaderType::Infer, ShaderLang::Pssl, shader_data, vec![]));
                        panic!("Unimplemented!");
                    },
                    "gdshad" => {
                        //parse as single stage
                        let mut vec = &shader_data;
                        let mut shader_data: ShaderData = ShaderData { data: Arc::new(Mutex::new(vec.clone())), compiled_data: None, descriptor: ShaderStageDescriptor::default() };
                        stages.push(ShaderStage::new(file_stem.clone(), ShaderType::Infer, ShaderLang::GodotShader, shader_data, vec![]));
                        panic!("Unimplemented!!");
                    }
                    _ => {
                        panic!("File is not a shader type!!");
                    }
                }
            }
            let mut stages_ptr = Vec::<ShaderPtr>::new();

            for stage in stages {
                unsafe {
                    stages_ptr.push(RenderPipelineSystem::register_shader_stage(stage));
                }
            }

            self.shader_name = file_name.clone();
            self.shader_stages = stages_ptr;
            self.asset_path = data.asset_path.clone();
        }
    }

    fn unload(&mut self) {
        todo!()
    }
}