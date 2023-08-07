pub mod entity;
pub mod physics;

use crate::common::engine::gamesys::{Base, Reflection};
use crate::common::components::entity::Entity;

pub trait BaseCompoent: Reflection{

    fn getEntity(&self) -> Box<Entity>;



}

