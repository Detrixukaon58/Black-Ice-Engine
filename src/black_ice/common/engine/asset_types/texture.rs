
use crate::black_ice::common::engine::{asset_mgr::*, asset_types::*};

pub struct Texture {

}

impl AssetResource for Texture {
    fn new() -> Self {
        Texture {  }
    }

    fn init(&mut self, data: InputData) {

    }

    fn update(&mut self) -> Result<OutputData, AssetResourceUpdateError> {

        Ok(OutputData::BYTEARRAY(vec![]))
    }

    fn unload(&mut self){

    }
}
