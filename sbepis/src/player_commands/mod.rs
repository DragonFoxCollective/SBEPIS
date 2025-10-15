use bevy_butler::*;

use crate::prelude::*;

use self::commands::*;
use self::notes::*;
use self::staff::*;

mod commands;
mod note_holder;
mod notes;
mod staff;

#[butler_plugin]
#[add_plugin(to_plugin = SbepisPlugin)]
pub struct PlayerCommandsPlugin;
