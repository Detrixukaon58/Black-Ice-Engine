pub mod entity;
pub mod physics;
pub mod component_system;

use crate::common::engine::gamesys::{Base, Reflection};
use crate::common::components::entity::Entity;

pub trait BaseComponent: Reflection{

    fn getEntity(&self) -> Box<Entity>;



}

