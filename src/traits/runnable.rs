use dyn_clone::DynClone;
use std::error::Error;
use std::fmt::Debug;
use std::collections::HashMap;

pub trait Runnable: DynClone + Debug {
    fn run(&self, aliases: &mut HashMap<String, String>) -> Result<String, Box<dyn Error>>;
}
dyn_clone::clone_trait_object!(Runnable);
