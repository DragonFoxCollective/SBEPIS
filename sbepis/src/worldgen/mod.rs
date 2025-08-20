use crate::prelude::*;
use bevy_butler::*;

pub mod desert;
pub mod low_lod;
pub mod terrain;

#[butler_plugin]
#[add_plugin(to_plugin = SbepisPlugin)]
pub struct WorldGenPlugin;
