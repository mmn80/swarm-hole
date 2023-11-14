use bevy::{app::PluginGroupBuilder, prelude::*};

use self::{
    laser::{LaserConfig, LaserPlugin},
    melee::{MeleeConfig, MeleePlugin},
};

pub mod laser;
pub mod melee;

pub struct SkillPluginGroup;

impl PluginGroup for SkillPluginGroup {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(SkillsPlugin)
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
    pub parent: Entity,
}
