use bevy_butler::*;
use name_tags::{AvailableNames, CandyMaterial};

pub mod consort;
pub mod imp;
pub mod name_tags;

#[butler_plugin]
#[add_plugin(to_plugin = crate::SbepisPlugin)]
pub struct NpcPlugin;

#[add_plugin(to_plugin = NpcPlugin, generics = <AvailableNames>, init = RonAssetPlugin::<AvailableNames>::new(&["names.ron"]))]
use bevy_common_assets::ron::RonAssetPlugin;

#[add_plugin(to_plugin = NpcPlugin, generics = <CandyMaterial>, init = MaterialPlugin::<CandyMaterial>::default())]
use bevy::pbr::MaterialPlugin;
