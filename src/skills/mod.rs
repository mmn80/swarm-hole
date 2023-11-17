use bevy::{app::PluginGroupBuilder, prelude::*};

use self::{
    health::HealthPlugin,
    laser::{Laser, LaserPlugin},
    melee::{Melee, MeleePlugin},
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
            .add(HealthPlugin)
            .add(XpDropsPlugin)
            .add(LaserPlugin)
            .add(MeleePlugin)
    }
}

#[derive(Clone, Reflect)]
pub enum Skill {
    Melee(Vec<Melee>),
    Laser(Vec<Laser>),
}

pub fn init_skills(entity: Entity, skills: &Vec<Skill>, world: &mut World) {
    for skill in skills {
        let mut ent = world.entity_mut(entity);
        match skill {
            Skill::Melee(levels) => {
                ent.insert(levels[0].clone());
            }
            Skill::Laser(levels) => {
                ent.insert(levels[0].clone());
            }
        }
    }
}
