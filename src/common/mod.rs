use self::{engine::gamesys::*, vertex::Vec3};

pub mod vertex;
pub mod angles;
pub mod matrices;
pub mod mesh;
pub mod transform;
pub mod filesystem;
pub mod engine;
pub mod textures;
pub mod materials;
pub mod components;

impl Base for Vec<Vec3>{}
impl Base for Vec<f32>{}
impl Base for Vec<(i16,i16)>{}
impl Base for Vec<(i16,i16,i16)>{}
impl Base for Vec<i16>{}
impl Base for Vec<(i16, Vec3)> {}
impl Base for String {}

pub const APP_DIR: &str = "F:\\Rust\\Program 1";

pub fn concat_str(a: &str, b: &str) -> String {

    let mut aa = a.clone().to_string();
    aa.push_str(b);

    return aa;
}

pub trait New<T> {
    fn new() -> T;
}

mod resources{

    ///This is the type for RESID. This must be used so as to differentiate between i32 type and REsource ID.
    pub type ResID = i32;

    /// This is used to define a resource type. Add this as a property to any structs you want to represent as a system resource that can be accessed.
    /// In order for this to be visible to the game, you must add a this as a private property for whatever struct you create for this:
    /// 
    /// e.g.
    /// 
    ///     pub struct MyResource 
    ///     {
    ///         resource: &'static Resource,  // This must be created this way for the Game Engine to be able to create a resource
    ///         ...
    ///     }
    /// 
    /// Also, when you create a new `Resource`, you must implement the `ResourceTrait`:
    /// 
    /// e.g.
    /// 
    /// ```rust
    /// impl ResourceTrait for MyResource 
    /// {
    ///     fn GetResource(&self) -> &'static Resource // These must be implemented as so so that the resource can be accessed by the game
    ///     {
    ///         return &self.resource;
    ///     }
    ///     
    ///     fn SetResource(&self, _resource: &'static Resource)
    ///     {
    ///         self.resource = _resource;
    ///     }
    ///     ...
    /// }
    /// 
    /// ```
    #[derive(Clone)]
    pub struct Resource {
        
        _res_id: ResID,


    }

    // Implementation for Resource
    impl Resource {
    }

    /// This is used for implementing Resources.
    /// If you want the Game Engine to be able to access the resource, then you must implement this trait.
    /// This will provide a set of functions that you must implement so that the Game Engine can create and save Resources.
    pub trait ResourceTrait {

        fn get_resource() -> &'static Resource;
        fn set_resource(_resource: &'static Resource);

    }
}