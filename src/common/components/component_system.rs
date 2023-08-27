// TODO: Implement a component registration system to allow for component allocation for entities
#![allow(unused)]
#![allow(non_snake_case)]
use std::{sync::*, fmt::{Display, Pointer}};

use crate::common::{engine::gamesys::*, components::entity::*, vertex::Vec3, matrices::{Matrix33, Matrix34, Vec4}};

use super::entity::entity_system::EntityID;

use crate::common::angles::{Ang3, Quat};
use serde::*;


pub struct ComponentSystem {
    component_register: Box<Vec<(entity_system::EntityID, Vec<ComponentRef<dyn BaseComponent>>)>>,
    constructor_register: Box<Vec<(std::any::TypeId, &'static (dyn Fn() -> Option<&'static dyn Base> + Sync))>>,
}

// TODO: Implement a way of reflecting components (need to complent component system first)
pub type ComponentRef<T> = Arc<Mutex<T>>;

pub fn ComponentRef_new<T>(item: T) -> ComponentRef<T> {
    return Arc::new(Mutex::new(item));
}

#[derive(Clone)]
pub enum Value {
    Null,
    Vec3(Vec3),
    Vec4(Vec4),
    Ang3(Ang3),
    Quat(Quat),
    Mat33(Matrix33),
    Mat34(Matrix34),
    I32(i32),
    F32(f32),
    String(String),
    Array(Vec<Value>),
    Component(String, Arc<Value>),
    
}

impl Display for Value {
    fn fmt(&self, f: &mut __private::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Null => writeln!(f, ""),
            Self::Ang3(ypr) => writeln!(f, "Ang3({y},{p},{r})", y = ypr.y, p = ypr.p, r = ypr.r),
            Self::Array(arr) => arr.fmt(f),
            Self::Component(name, v) => writeln!(f, "\"{name}\":{value}", value=format!("{}", v)),
            Self::F32(v) => writeln!(f, "{}", v),
            Self::I32(v) => writeln!(f, "{}", v),
            Self::Mat33(mat33) => writeln!(f, "Mat33({x}, {y}, {z})",
            x = mat33.x,
            y = mat33.y,
            z = mat33.z),
            Self::Mat34(mat34) => writeln!(f, "Mat34({x}, {y}, {z})",
            x = mat34.x,
            y = mat34.y,
            z = mat34.z),
            Self::Quat(q) => writeln!(f, "Quat({x}, {y}, {z}, {w}", x=q.x, y=q.y, z=q.z, w=q.w),
            Self::String(s) => writeln!(f, "\"{}\"", s),
            Self::Vec3(v) => writeln!(f, "Vec3({x}, {y}, {z})", x=v.x, y=v.y, z=v.z),
            Self::Vec4(v) => writeln!(f, "Vec4({x}, {y}, {z}, {w})", x=v.x, y=v.y, z=v.z, w=v.w),
            
        }
    }
}

impl std::ops::Index<&'static str> for Value {
    type Output = Arc<Value>;

    fn index(&self, index: &'static str) -> &Self::Output {
        match self.get(index) {
            Some(v) => v,
            None => panic!("No such {}", index)
        }
    }
}

impl Value {
    pub fn get_s(&self, index: String) -> Option<&Arc<Value>> {
        match self {
            Value::Component(name, v) => {
                if *name == index {
                    Some(v)
                }
                else{
                    None
                }
            },
            Value::Array(arr) =>{
                for v in arr {
                    match v {
                        Value::Component(name, v) =>{
                            if *name == index {
                                return Some(v);
                            }
                        },
                        _ => continue
                    }
                }

                None
            },
            _ => {None}
        }
    }

    pub fn get(&self, index: &'static str) -> Option<&Arc<Value>> 
    {
        self.get_s(String::from(index))
    }

    pub fn as_component(&self) -> Option<(String, Arc<Value>)>
    {
        match self {
            Value::Component(name, v) => Some((name.clone(), v.clone())),
            _ => None
        }
    }

    pub fn as_str(&self) -> Option<String> {
        match self {
            Value::String(s) => Some(s.clone()),
            _ => None
        }
    }

    pub fn as_vec3(&self) -> Option<Vec3> {
        match self {
            Value::Vec3(s) => Some(s.clone()),
            _ => None
        }
    }

    pub fn as_quat(&self) -> Option<Quat> {
        match self {
            Value::Quat(s) => Some(s.clone()),
            _ => None
        }
    }

}

pub struct ValueBuilder {
    inner: Option<Value>
}

impl ValueBuilder {

    pub fn new() -> Self {
        Self { inner: Some(Value::Null) }
    }

    pub fn build(&self) -> Value {
        self.inner.as_ref().expect("Failed to build value!!!").clone()
    }

    pub fn from_str(&mut self, json: &str) -> &mut Self 
    {

        fn tokenize(json: &str) -> Vec<String>{

            let mut tokens: Vec<String> = Vec::new();
            let mut token = String::new();
            let mut json_begin = 0;
            let mut arr_begin = 0;
            let mut quote_begin = false;
            for mut i in 0..json.len() {
                let char = json.chars().nth(i).unwrap();
                
                if !quote_begin {
                    if char == '{' {
                        // Json Begin
                        if token.is_empty() {
                            token.push(char);
                            tokens.push(token);
                        }
                        else {
                            tokens.push(token);
                            token = String::from(char);
                            tokens.push(token);
                        }
                        
                        token = String::new();
                        json_begin += 1;
                        continue;
                    }
                    if char == '}' && json_begin > 0{
                        
                        if token.is_empty() {
                            token.push(char);
                            tokens.push(token);
                        }
                        else {
                            tokens.push(token);
                            token = String::from(char);
                            tokens.push(token);
                        }
                        token = String::new();
                        json_begin -= 1;
                        continue;
                    }

                    if char == '[' {
                        if token.is_empty() {
                            token.push(char);
                            tokens.push(token);
                        }
                        else {
                            tokens.push(token);
                            token = String::from(char);
                            tokens.push(token);
                        }
                        token = String::new();
                        arr_begin += 1;
                        continue;
                    }
                    if char == ']' && arr_begin > 0{
                        
                        if token.is_empty() {
                            token.push(char);
                            tokens.push(token);
                        }
                        else {
                            tokens.push(token);
                            token = String::from(char);
                            tokens.push(token);
                        }
                        token = String::new();
                        arr_begin -= 1;
                        continue;
                    }

                    if char == ',' {
                        tokens.push(token);
                        token = String::new();
                        continue;
                    }

                    if char == ':' {
                        if token.is_empty() {
                            token.push(char);
                            tokens.push(token);
                        }
                        else {
                            tokens.push(token);
                            token = String::from(char);
                            tokens.push(token);
                        }
                        token = String::new();
                        continue;
                    }

                    if char == '(' {
                        if token.is_empty() {
                            token.push(char);
                            tokens.push(token);
                        }
                        else {
                            tokens.push(token);
                            token = String::from(char);
                            tokens.push(token);
                        }
                        token = String::new();
                        continue;
                    }

                    if char == ')' {
                        if token.is_empty() {
                            token.push(char);
                            tokens.push(token);
                        }
                        else {
                            tokens.push(token);
                            token = String::from(char);
                            tokens.push(token);
                        }
                        token = String::new();
                        continue;
                    }
                    
                }
                else{
                    if char == '\\' && i + 1 < json.len(){
                        let next_char = json.chars().nth(i+1).unwrap();

                        match next_char {
                            '\\' => {
                                // is just a single \
                                token.push('\\');
                                i = i + 1;
                            },
                            '"' => {
                                // is just "
                                token.push('"');
                                i = i + 1;
                            },
                            'n' => {
                                // new line \n
                                token.push('\n');
                                i = i + 1;
                            },
                            _ => {
                                // just add both characters to the tokentoekns
                                token.push('\\');
                                token.push(next_char);
                                i = i+1;
                            }
                        }
                        continue;
                    }
                }

                if char == '"' {
                    //token = format!("{}{}", token, char);
                    if quote_begin {
                        tokens.push(token);
                        token = String::new();
                    }
                    quote_begin = !quote_begin;
                    continue;
                }

                
                token.push(char);
                



            }

            tokens
        }
        // do a for loop
        // let mut stack: Vec<(usize, usize)> = Vec::new();

        // let mut start: usize = 0;
        // let mut end: usize = stack.len();

        // 'run: loop {

        //     let st = &json[start..end];

        //     // process this part of the string

        //     // Check if current line is also a json
        //     let lines = st.split("\n").collect::<Vec<&str>>();
        //     let current_line = lines[0];

        //     if stack.is_empty() {
        //         break 'run;
        //     }
        //     if start == end {
        //         (start, end) = stack.pop().unwrap();
        //     }
        // }
        fn make_value(tokens: Vec<String>) -> Value {
            let mut json = 0;// number of { encountered without } to close
            let mut json_starts: Vec<usize> = Vec::new();
            let mut arr = 0;// number of [ encountered without ] to close
            let mut current_arr = 0;
            let mut arr_values: Vec<Vec<Value>> = Vec::new();
            let mut ret: Value = Value::Null;
            loop{
                let mut i = 0;
                match tokens[i].as_str() {
                    "{" => {
                        json += 1;
                        json_starts.push(i);
                    }

                    "}" => {
                        // recurse this to get out some json value
                        let start = json_starts.pop().unwrap();
                        let value = make_value(tokens[start + 1..i].to_vec());
                        if arr > 0 {
                            arr_values[current_arr].push(value);
                        }
                        else {
                            ret = value;
                            break;
                        }
                    }

                    "[" => {
                        // treat this like start of a list
                        if json == 0 {
                            arr += 1;
                            arr_values.push(Vec::new());
                            current_arr = arr_values.len() - 1;
                        }
                    }

                    "]" => {
                        // treat this like end of a list
                        if json == 0{
                            arr -= 1;
                            if arr > 0 {
                                let arr_val = arr_values.pop().unwrap();
                                current_arr = arr_values.len() - 1;
                                arr_values[current_arr].push(Value::Array(arr_val));
                            }
                            else
                            {
                                ret = Value::Array(arr_values.pop().unwrap());
                                break;
                            }
                        }
                    }

                    "(" => {
                        if json == 0 {
                            match tokens[i-1].as_str() {
                                "Vec4" => {},
                                "Vec3" => {},
                                _ => {}
                            }
                        }
                    }

                    ")" => {
                        if json == 0 {

                        }
                    }

                    ":" => {
                        if json == 0 {
                            
                        }
                    }

                    _ => {
                        if json == 0 {
                            
                        }
                    }

                }
                i += 1;
            }

            ret
        }
        let tokens = tokenize(json);
        println!("{:?}", tokens);
        self
    }
}

pub type ConstructorDefinition = Arc<Value>;

pub trait Constructor<T> where T: Base {
    unsafe fn construct(entity: ComponentRef<entity_system::Entity>, definition: &ConstructorDefinition) -> Option<ComponentRef<T>>;
}

pub trait BaseComponent: Reflection + Send{
    fn get_entity(&self) -> ComponentRef<entity_system::Entity>;
    fn process_event(&mut self, event: &entity_system::entity_event::Event);
    fn get_event_mask(&self) -> entity_system::entity_event::EventFlag;
}

impl ComponentSystem {

    pub fn new() -> ComponentSystem {
        ComponentSystem { 
            component_register: Box::new(Vec::new()),
            constructor_register: Box::new(Vec::new()),
        }
    }

    pub fn entity_add_component(&mut self, entity: EntityID, component: ComponentRef<dyn BaseComponent>){
        println!("Adding component!!");
        let register = self.component_register.as_mut();
        for (entity_id, mut vec) in register.to_vec()
        {
            if entity_id.eq(&entity) {
                vec.push(component);
                return;
            }
        }
        register.push((entity, vec![component]));
        
    }

    pub fn entity_get_components(&mut self, entity: EntityID) -> Vec<ComponentRef<dyn BaseComponent>> {

        let register = self.component_register.to_vec();
        for (entity_id, vec) in register {
            if entity_id.eq(&entity) {
                return vec.clone();
            }
        }

        return vec![];

    }


    pub fn init(&'static self){

        
    }

    pub fn processing(&'static self) -> i32 {



        0
    }
}