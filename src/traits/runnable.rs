use dyn_clone::DynClone;
use std::error::Error;

pub trait Runnable: DynClone {
    fn run(&self) -> Result<String, Box<dyn Error>>;
}
dyn_clone::clone_trait_object!(Runnable);
