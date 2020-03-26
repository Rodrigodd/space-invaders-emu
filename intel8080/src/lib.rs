pub mod dissasembler;
pub mod interpreter;
mod intel8080;

mod write_adapter;

pub use intel8080::*;