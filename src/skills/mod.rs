use bevy::{app::PluginGroupBuilder, prelude::*};

use self::{health::HealthPlugin, laser::LaserPlugin, melee::MeleePlugin, xp_drops::XpDropsPlugin};

pub mod health;
pub mod laser;
pub mod melee;
pub mod xp_drops;

pub struct SkillPluginGroup;

impl PluginGroup for SkillPluginGroup {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(HealthPlugin)
            .add(XpDropsPlugin)
            .add(LaserPlugin)
            .add(MeleePlugin)
    }
}
