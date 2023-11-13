use bevy::{app::PluginGroupBuilder, prelude::*};

use self::{laser::LaserPlugin, melee::MeleePlugin};

pub mod laser;
pub mod melee;

pub struct WeaponPlugins;

impl PluginGroup for WeaponPlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(LaserPlugin)
            .add(MeleePlugin)
    }
}
