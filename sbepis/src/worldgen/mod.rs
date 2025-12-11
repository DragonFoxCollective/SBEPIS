use crate::prelude::*;
use bevy_auto_plugin::prelude::*;

pub mod desert;
pub mod terrain;

#[derive(AutoPlugin)]
#[auto_add_plugin(plugin = SbepisPlugin)]
#[auto_plugin(impl_plugin_trait)]
pub struct WorldGenPlugin;
