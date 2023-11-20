use bevy::{
    app::PluginGroupBuilder,
    asset::{io::Reader, AssetLoader, AsyncReadExt, LoadContext},
    ecs::system::Command,
    prelude::*,
    utils::{thiserror, thiserror::Error, BoxedFuture},
};
use rand::prelude::*;
use serde::Deserialize;

use crate::app::AppState;

use self::{
    health::{HealthPlugin, HealthRegen, HealthRegenBoost, MaxHealth, MaxHealthBoost},
    laser::{Laser, LaserPlugin},
    melee::{Melee, MeleePlugin},
    xp::{XpDropsPlugin, XpGather, XpGatherBoost, XpGatherState},
};

pub mod health;
pub mod laser;
pub mod melee;
pub mod xp;

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
        app.register_type::<Skill>()
            .init_asset::<SkillsAsset>()
            .init_asset_loader::<SkillsAssetLoader>()
            .init_resource::<SkillsAssetHandle>()
            .init_resource::<SkillUpgradeOptions>()
            .add_systems(Startup, setup_skills_asset_handle)
            .add_systems(Update, hot_reload_equipped_skills)
            .add_systems(Update, init_upgrade_menu.run_if(in_state(AppState::Run)))
            .add_systems(
                Update,
                apply_upgrade_selection.run_if(in_state(AppState::Upgrade)),
            );
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Copy, Clone, Reflect, Debug, Deserialize)]
pub enum Skill {
    Health,
    HealthBoost,
    HealthRegen,
    HealthRegenBoost,
    XpGather,
    XpGatherBoost,
    Melee,
    Laser,
}

#[derive(Clone, Reflect, Debug, Deserialize)]
pub struct Skills {
    pub health: Option<Vec<MaxHealth>>,
    pub health_boost: Option<Vec<MaxHealthBoost>>,
    pub health_regen: Option<Vec<HealthRegen>>,
    pub health_regen_boost: Option<Vec<HealthRegenBoost>>,
    pub xp_gather: Option<Vec<XpGather>>,
    pub xp_gather_boost: Option<Vec<XpGatherBoost>>,
    pub melee: Option<Vec<Melee>>,
    pub laser: Option<Vec<Laser>>,
}

impl Skills {
    //TODO: make macro
    pub fn get_max_skill_levels(&self) -> Vec<EquippedSkill> {
        let mut res = vec![];
        if let Some(levels) = &self.health {
            res.push(EquippedSkill {
                skill: Skill::Health,
                level: levels.len() as u8,
            });
        }
        if let Some(levels) = &self.health_boost {
            res.push(EquippedSkill {
                skill: Skill::HealthBoost,
                level: levels.len() as u8,
            });
        }
        if let Some(levels) = &self.health_regen {
            res.push(EquippedSkill {
                skill: Skill::HealthRegen,
                level: levels.len() as u8,
            });
        }
        if let Some(levels) = &self.health_regen_boost {
            res.push(EquippedSkill {
                skill: Skill::HealthRegenBoost,
                level: levels.len() as u8,
            });
        }
        if let Some(levels) = &self.xp_gather {
            res.push(EquippedSkill {
                skill: Skill::XpGather,
                level: levels.len() as u8,
            });
        }
        if let Some(levels) = &self.xp_gather_boost {
            res.push(EquippedSkill {
                skill: Skill::XpGatherBoost,
                level: levels.len() as u8,
            });
        }
        if let Some(levels) = &self.melee {
            res.push(EquippedSkill {
                skill: Skill::Melee,
                level: levels.len() as u8,
            });
        }
        if let Some(levels) = &self.laser {
            res.push(EquippedSkill {
                skill: Skill::Laser,
                level: levels.len() as u8,
            });
        }
        res
    }

    //TODO: make macro
    pub fn insert_components(&self, entity: Entity, world: &mut World) {
        let Some(mut ent) = world.get_entity_mut(entity) else {
            return;
        };
        let (had_equipped, mut equipped) = {
            if let Some(equipped) = ent.get::<EquippedSkills>() {
                (true, equipped.clone())
            } else {
                (false, EquippedSkills(vec![]))
            }
        };
        {
            let skill = Skill::Health;
            let sk = equipped.get(skill);
            if !had_equipped || sk.is_some() {
                if let Some(levels) = &self.health {
                    let sk = sk.unwrap_or(EquippedSkill { skill, level: 0 });
                    ent.insert(levels[sk.level as usize].clone());
                    equipped.update_skill(sk);
                }
            }
        }
        {
            let skill = Skill::HealthBoost;
            let sk = equipped.get(skill);
            if !had_equipped || sk.is_some() {
                if let Some(levels) = &self.health_boost {
                    let sk = sk.unwrap_or(EquippedSkill { skill, level: 0 });
                    ent.insert(levels[sk.level as usize].clone());
                    equipped.update_skill(sk);
                }
            }
        }
        {
            let skill = Skill::HealthRegen;
            let sk = equipped.get(skill);
            if !had_equipped || sk.is_some() {
                if let Some(levels) = &self.health_regen {
                    let sk = sk.unwrap_or(EquippedSkill { skill, level: 0 });
                    ent.insert(levels[sk.level as usize].clone());
                    equipped.update_skill(sk);
                }
            }
        }
        {
            let skill = Skill::HealthRegenBoost;
            let sk = equipped.get(skill);
            if !had_equipped || sk.is_some() {
                if let Some(levels) = &self.health_regen_boost {
                    let sk = sk.unwrap_or(EquippedSkill { skill, level: 0 });
                    ent.insert(levels[sk.level as usize].clone());
                    equipped.update_skill(sk);
                }
            }
        }
        {
            let skill = Skill::XpGather;
            let sk = equipped.get(skill);
            if !had_equipped || sk.is_some() {
                if let Some(levels) = &self.xp_gather {
                    let sk = sk.unwrap_or(EquippedSkill { skill, level: 0 });
                    ent.insert(levels[sk.level as usize].clone());
                    equipped.update_skill(sk);
                }
            }
        }
        {
            let skill = Skill::XpGatherBoost;
            let sk = equipped.get(skill);
            if !had_equipped || sk.is_some() {
                if let Some(levels) = &self.xp_gather_boost {
                    let sk = sk.unwrap_or(EquippedSkill { skill, level: 0 });
                    ent.insert(levels[sk.level as usize].clone());
                    equipped.update_skill(sk);
                }
            }
        }
        {
            let skill = Skill::Melee;
            let sk = equipped.get(skill);
            if !had_equipped || sk.is_some() {
                if let Some(levels) = &self.melee {
                    let sk = sk.unwrap_or(EquippedSkill { skill, level: 0 });
                    ent.insert(levels[sk.level as usize].clone());
                    equipped.update_skill(sk);
                }
            }
        }
        {
            let skill = Skill::Laser;
            let sk = equipped.get(skill);
            if !had_equipped || sk.is_some() {
                if let Some(levels) = &self.laser {
                    let sk = sk.unwrap_or(EquippedSkill { skill, level: 0 });
                    ent.insert(levels[sk.level as usize].clone());
                    equipped.update_skill(sk);
                }
            }
        }
        ent.insert(equipped);
    }
}

pub struct UpdateSkillComponents {
    pub entity: Entity,
    pub skills: Skills,
}

impl Command for UpdateSkillComponents {
    fn apply(self, world: &mut World) {
        self.skills.insert_components(self.entity, world);
    }
}

#[derive(Copy, Clone)]
pub struct EquippedSkill {
    pub skill: Skill,
    level: u8,
}

impl EquippedSkill {
    pub fn new(skill: Skill) -> Self {
        Self { skill, level: 0 }
    }

    pub fn get_level(&self) -> u8 {
        self.level + 1
    }
}

#[derive(Component, Clone)]
pub struct EquippedSkills(pub Vec<EquippedSkill>);

impl EquippedSkills {
    pub fn get(&self, skill: Skill) -> Option<EquippedSkill> {
        self.0.iter().find(|s| s.skill == skill).map(|s| *s)
    }

    pub fn update_skill(&mut self, skill: EquippedSkill) {
        let mut found = false;
        for equipped in &mut self.0 {
            if equipped.skill == skill.skill {
                equipped.level = skill.level;
                found = true;
                break;
            }
        }
        if !found {
            self.0.push(skill);
        }
    }
}

#[derive(Component)]
pub struct HotReloadEquippedSkills;

fn hot_reload_equipped_skills(
    skills_asset_handle: Res<SkillsAssetHandle>,
    skills_asset: Res<Assets<SkillsAsset>>,
    mut skills_asset_events: EventReader<AssetEvent<SkillsAsset>>,
    q_equipped: Query<Entity, With<HotReloadEquippedSkills>>,
    mut cmd: Commands,
) {
    for ev in skills_asset_events.read() {
        let h = skills_asset_handle.0.clone();
        if ev.is_loaded_with_dependencies(&h) {
            if let Some(skills_asset) = skills_asset.get(h) {
                for entity in &q_equipped {
                    cmd.add(UpdateSkillComponents {
                        entity,
                        skills: skills_asset.skills.clone(),
                    });
                }
            }
        }
    }
}

#[derive(Component)]
pub struct MaxUpgradableSkills(pub u8);

#[derive(Resource, Default)]
pub struct SkillUpgradeOptions {
    pub entity: Option<Entity>,
    pub skills: Vec<EquippedSkill>,
    pub selected: Option<EquippedSkill>,
}

fn init_upgrade_menu(
    mut next_state: ResMut<NextState<AppState>>,
    mut upgrades: ResMut<SkillUpgradeOptions>,
    skills_asset_handle: Res<SkillsAssetHandle>,
    skills_asset: Res<Assets<SkillsAsset>>,
    q_xp_gather_state: Query<(Entity, &XpGatherState, &MaxUpgradableSkills)>,
    q_equipped_skills: Query<&EquippedSkills>,
) {
    for (entity, xp_gather_state, max_skills) in &q_xp_gather_state {
        if xp_gather_state.get_gather_level() > xp_gather_state.get_player_level() {
            if let Ok(equipped_skills) = q_equipped_skills.get(entity) {
                let mut skill_upgrades = vec![];
                if let Some(skills_asset) = skills_asset.get(&skills_asset_handle.0) {
                    let all_equipped = equipped_skills.0.clone();
                    let max_levels = skills_asset.skills.get_max_skill_levels();
                    let mut all_equipped_count = 0;
                    for skill in &all_equipped {
                        if let Some(max_skill) = max_levels.iter().find(|s| s.skill == skill.skill)
                        {
                            all_equipped_count += 1;
                            if skill.level < max_skill.level - 1 {
                                skill_upgrades.push(skill.clone());
                            }
                        }
                    }
                    for skill in &mut skill_upgrades {
                        skill.level += 1;
                    }
                    if all_equipped_count < max_skills.0 as usize {
                        for mut skill in max_levels {
                            if all_equipped
                                .iter()
                                .find(|s| s.skill == skill.skill)
                                .is_none()
                            {
                                skill.level = 0;
                                skill_upgrades.push(skill);
                            }
                        }
                    }
                }
                upgrades.skills.clear();
                let mut rng = thread_rng();
                upgrades.skills = skill_upgrades
                    .choose_multiple(&mut rng, 3)
                    .map(|s| s.clone())
                    .collect();
                if !upgrades.skills.is_empty() {
                    upgrades.entity = Some(entity);
                    upgrades.selected = None;
                    next_state.set(AppState::Upgrade);
                }
            }
        } else {
            upgrades.entity = None;
            upgrades.skills.clear();
            upgrades.selected = None;
        }
    }
}

fn apply_upgrade_selection(
    mut next_state: ResMut<NextState<AppState>>,
    mut upgrades: ResMut<SkillUpgradeOptions>,
    skills_asset_handle: Res<SkillsAssetHandle>,
    skills_asset: Res<Assets<SkillsAsset>>,
    mut q_xp_gather_state: Query<&mut XpGatherState>,
    mut q_equipped_skills: Query<&mut EquippedSkills>,
    mut cmd: Commands,
) {
    let (Some(entity), Some(selected)) = (upgrades.entity, upgrades.selected) else {
        return;
    };
    if let Ok(mut xp_gather_state) = q_xp_gather_state.get_mut(entity) {
        xp_gather_state.upgrade_player_level();
    }
    if let Ok(mut equipped_skills) = q_equipped_skills.get_mut(entity) {
        equipped_skills.update_skill(selected);
    }
    if let Some(skills_asset) = skills_asset.get(&skills_asset_handle.0) {
        cmd.add(UpdateSkillComponents {
            entity,
            skills: skills_asset.skills.clone(),
        });
    }
    upgrades.entity = None;
    upgrades.skills.clear();
    upgrades.selected = None;
    next_state.set(AppState::Run);
}

#[derive(Resource, Default)]
pub struct SkillsAssetHandle(pub Handle<SkillsAsset>);

fn setup_skills_asset_handle(
    mut skills_asset_handle: ResMut<SkillsAssetHandle>,
    asset_server: Res<AssetServer>,
) {
    skills_asset_handle.0 = asset_server.load("all.skills.ron");
}

#[derive(Debug, Deserialize)]
pub struct SkillUiConfig {
    pub skill: Skill,
    pub name: String,
}

#[derive(Asset, TypePath, Debug, Deserialize)]
pub struct SkillsAsset {
    pub ui_config: Vec<SkillUiConfig>,
    pub skills: Skills,
}

#[derive(Default)]
pub struct SkillsAssetLoader;

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum SkillsAssetLoaderError {
    #[error("Could not load asset: {0}")]
    Io(#[from] std::io::Error),
    #[error("Could not parse RON: {0}")]
    RonSpannedError(#[from] ron::error::SpannedError),
}

impl AssetLoader for SkillsAssetLoader {
    type Asset = SkillsAsset;
    type Settings = ();
    type Error = SkillsAssetLoaderError;
    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        _settings: &'a (),
        _load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;
            let custom_asset = ron::de::from_bytes::<SkillsAsset>(&bytes)?;
            Ok(custom_asset)
        })
    }

    fn extensions(&self) -> &[&str] {
        &["skills.ron"]
    }
}
