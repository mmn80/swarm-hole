use bevy::{app::PluginGroupBuilder, prelude::*};

use self::{
    health::HealthPlugin,
    laser::{LaserConfig, LaserPlugin},
    melee::{MeleeConfig, MeleePlugin},
    xp_drops::XpDropsPlugin,
};

pub mod health;
pub mod laser;
pub mod melee;
pub mod xp_drops;

pub struct SkillPluginGroup;

impl PluginGroup for SkillPluginGroup {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(SkillsPlugin)
            .add(HealthPlugin)
            .add(XpDropsPlugin)
            .add(LaserPlugin)
            .add(MeleePlugin)
    }
}

pub struct SkillsPlugin;

impl Plugin for SkillsPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<AddSkillEvent>();
    }
}

#[derive(Copy, Clone, Reflect)]
pub enum Skill {
    Melee(MeleeConfig),
    Laser(LaserConfig),
}

#[derive(Event)]
pub struct AddSkillEvent {
    pub skill: Skill,
    pub agent: Entity,
}
