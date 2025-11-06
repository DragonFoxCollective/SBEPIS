use bevy_butler::*;

use crate::prelude::*;

pub mod consort;
pub mod imp;
pub mod name_tags;

#[butler_plugin]
#[add_plugin(to_plugin = SbepisPlugin)]
pub struct NpcPlugin;
