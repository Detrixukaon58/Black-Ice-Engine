
use std::sync::Arc;
use std::{fmt, error::Error};
use crate::black_ice::common::engine::asset_mgr::AssetData;

pub mod shader_asset;

pub enum InputData {
    OBJECT(String, Arc<InputData>),
    ARRAY(Vec<InputData>),
    BYTEARRAY(i32),// This is a pointer to the loaded byte data so as to ensure that shared
    // references do not have multiple copies of the same data throughout memory
}

pub enum OutputData {
    BYTEARRAY(Vec<i8>),
    INT(i32),
    FLOAT(f32),
    STRING(String),
    NONE
}

#[derive(Debug)]
pub struct AssetResourceUpdateError {}

impl fmt::Display for AssetResourceUpdateError {

    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Failed to update asset data!!")
    }

}

impl Error for AssetResourceUpdateError {}


pub trait AssetResource {

    fn new() -> Self;

    fn init(&mut self, data: Arc<AssetData>); // loads the asset's data. This needs to be defined in order for the
    // asset manager to be able to process your custom asset resource

    fn update(&mut self) -> Result<OutputData, AssetResourceUpdateError>{
        return Ok(OutputData::NONE);
    }// possible update function
    // for any asset that mey need to stream data instead of loading just the once

    fn unload(&mut self);// This must be done in order for the asset data that has been loaded
    // to be reset on the case of the asset no longer being used or for the application to be
    // closed. You must remember that data that has been previously loaded will be stored in a
    // shared memory in order to be memory efficient. So when loading and unloading, you must make
    // sure that you only drop any data that you have created in the load function and not data
    // that has been passed into the load function as a parameter

    
}


pub mod texture;
