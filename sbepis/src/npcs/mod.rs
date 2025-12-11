use bevy_auto_plugin::prelude::*;

use crate::prelude::*;

pub mod consort;
pub mod imp;
pub mod name_tags;

#[derive(AutoPlugin)]
#[auto_add_plugin(plugin = SbepisPlugin)]
#[auto_plugin(impl_plugin_trait)]
pub struct NpcPlugin;
