// TODO: Implement a component registration system to allow for component allocation for entities

use std::sync::*;

struct ComponentSystem {

}

// TODO: Implement a way of reflecting components (need to complent component system first)
type ComponentRef<T> = Arc<Mutex<T>>;