use dyn_clone::DynClone;
use std::error::Error;
use std::fmt::Debug;

pub trait Runnable: DynClone + Debug {
    fn run(&self) -> Result<String, Box<dyn Error>>;
}
dyn_clone::clone_trait_object!(Runnable);
