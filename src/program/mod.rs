pub mod val_program;

use glium::{Texture2d, Display};
use yaml_rust::Yaml;

pub trait Program {
    fn from_yaml(doc: &Yaml, display: &Display) -> Self;
    fn step(&self, board: &mut Texture2d);
    fn get_dimensions(&self) -> (u32, u32);
}