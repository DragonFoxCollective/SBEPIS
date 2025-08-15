use bevy_butler::*;

pub mod low_lod;
pub mod terrain;

#[butler_plugin]
#[add_plugin(to_plugin = crate::SbepisPlugin)]
pub struct WorldGenPlugin;
