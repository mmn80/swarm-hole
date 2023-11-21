use std::fmt;

use bevy::{
    app::PluginGroupBuilder,
    asset::{io::Reader, AssetLoader, AsyncReadExt, LoadContext},
    ecs::system::Command,
    prelude::*,
    utils::{thiserror, thiserror::Error, BoxedFuture, HashMap, HashSet},
};
use rand::prelude::*;
use serde::Deserialize;

use crate::app::AppState;

use self::{
    health::{HealthPlugin, HealthRegen, MaxHealth},
    laser::{Laser, LaserPlugin},
    melee::{Melee, MeleePlugin},
    xp::{XpGather, XpGatherState, XpPlugin},
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
            .add(XpPlugin)
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

#[derive(PartialEq, Eq, PartialOrd, Ord, Copy, Clone, Reflect, Debug, Deserialize, Hash)]
pub enum Skill {
    Health,
    HealthRegen,
    XpGather,
    Melee,
    Laser,
}

#[derive(Clone, Reflect, Debug, Deserialize)]
pub struct Skills {
    pub health: Option<Vec<MaxHealth>>,
    pub health_regen: Option<Vec<HealthRegen>>,
    pub xp_gather: Option<Vec<XpGather>>,
    pub melee: Option<Vec<Melee>>,
    pub laser: Option<Vec<Laser>>,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Default, Debug, Reflect, Deserialize)]
pub struct Level(u8);

impl Level {
    pub fn is_first(&self) -> bool {
        self.0 == 0
    }

    pub fn is_last(&self, levels_count: usize) -> bool {
        self.0 as usize >= levels_count - 1
    }

    pub fn next(&self) -> Self {
        Self(self.0 + 1)
    }

    pub fn prev(&self) -> Option<Self> {
        if self.0 == 0 {
            None
        } else {
            Some(Self(self.0 - 1))
        }
    }

    pub fn index<'a, T>(&'a self, list: &'a Vec<T>) -> Option<&T> {
        list.get(self.0 as usize)
    }
}

impl fmt::Display for Level {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0 + 1)
    }
}

pub struct ReflectedSkill {
    pub skill: Skill,
    pub levels: Vec<Box<dyn Reflect>>,
}

impl ReflectedSkill {
    pub fn new(skill: Skill, levels: Vec<Box<dyn Reflect>>) -> Self {
        Self { skill, levels }
    }
}

impl Skills {
    //TODO: make macro
    pub fn get_reflected(&self) -> HashMap<Skill, Vec<Box<dyn Struct>>> {
        let mut res = HashMap::new();
        if let Some(levels) = &self.health {
            res.insert(
                Skill::Health,
                levels
                    .iter()
                    .map(|s| Box::new(s.clone()) as Box<dyn Struct>)
                    .collect(),
            );
        }
        if let Some(levels) = &self.health_regen {
            res.insert(
                Skill::HealthRegen,
                levels
                    .iter()
                    .map(|s| Box::new(s.clone()) as Box<dyn Struct>)
                    .collect(),
            );
        }
        if let Some(levels) = &self.xp_gather {
            res.insert(
                Skill::XpGather,
                levels
                    .iter()
                    .map(|s| Box::new(s.clone()) as Box<dyn Struct>)
                    .collect(),
            );
        }
        if let Some(levels) = &self.melee {
            res.insert(
                Skill::Melee,
                levels
                    .iter()
                    .map(|s| Box::new(s.clone()) as Box<dyn Struct>)
                    .collect(),
            );
        }
        if let Some(levels) = &self.laser {
            res.insert(
                Skill::Laser,
                levels
                    .iter()
                    .map(|s| Box::new(s.clone()) as Box<dyn Struct>)
                    .collect(),
            );
        }
        res
    }

    pub fn parse_numeric_fields(refl: &Box<dyn Struct>) -> HashMap<String, f32> {
        let mut res = HashMap::new();
        for fld_idx in 0..refl.field_len() {
            if let (Some(fln_name), Some(fld_val)) = (refl.name_at(fld_idx), refl.field_at(fld_idx))
            {
                let mut f32_val = fld_val.downcast_ref::<f32>().map(|n| *n);
                if f32_val.is_none() {
                    if let Some(n) = fld_val.downcast_ref::<u32>() {
                        f32_val = Some(*n as f32);
                    }
                }
                if let Some(num) = f32_val {
                    res.insert(fln_name.to_string(), num);
                }
            }
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
                (false, EquippedSkills::default())
            }
        };
        {
            let skill = Skill::Health;
            let lvl = equipped.get_level(skill);
            if !had_equipped || lvl.is_some() {
                if let Some(levels) = &self.health {
                    let lvl = lvl.unwrap_or_default();
                    ent.insert(lvl.index(levels).unwrap().clone());
                    equipped.set_level(skill, lvl, false);
                }
            }
        }
        {
            let skill = Skill::HealthRegen;
            let lvl = equipped.get_level(skill);
            if !had_equipped || lvl.is_some() {
                if let Some(levels) = &self.health_regen {
                    let lvl = lvl.unwrap_or_default();
                    ent.insert(lvl.index(levels).unwrap().clone());
                    equipped.set_level(skill, lvl, false);
                }
            }
        }
        {
            let skill = Skill::XpGather;
            let lvl = equipped.get_level(skill);
            if !had_equipped || lvl.is_some() {
                if let Some(levels) = &self.xp_gather {
                    let lvl = lvl.unwrap_or_default();
                    ent.insert(lvl.index(levels).unwrap().clone());
                    equipped.set_level(skill, lvl, false);
                }
            }
        }
        {
            let skill = Skill::Melee;
            let lvl = equipped.get_level(skill);
            if !had_equipped || lvl.is_some() {
                if let Some(levels) = &self.melee {
                    let lvl = lvl.unwrap_or_default();
                    ent.insert(lvl.index(levels).unwrap().clone());
                    equipped.set_level(skill, lvl, false);
                }
            }
        }
        {
            let skill = Skill::Laser;
            let lvl = equipped.get_level(skill);
            if !had_equipped || lvl.is_some() {
                if let Some(levels) = &self.laser {
                    let lvl = lvl.unwrap_or_default();
                    ent.insert(lvl.index(levels).unwrap().clone());
                    equipped.set_level(skill, lvl, false);
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

#[derive(Component, Clone, Default)]
pub struct EquippedSkills {
    pub equipped: HashMap<Skill, Level>,
    pub selected: HashSet<Skill>,
}

impl EquippedSkills {
    pub fn new(pre_equipped: &Vec<(Skill, Level)>, pre_selected: &Vec<Skill>) -> Self {
        let mut equipped = HashMap::from_iter(pre_equipped.into_iter().cloned());
        let selected = HashSet::from_iter(pre_selected.into_iter().cloned());
        for skill in &selected {
            if !equipped.contains_key(skill) {
                equipped.insert(*skill, Level::default());
            }
        }
        Self { equipped, selected }
    }

    pub fn get_level(&self, skill: Skill) -> Option<Level> {
        self.equipped.get(&skill).copied()
    }

    pub fn set_level(&mut self, skill: Skill, level: Level, is_selected: bool) {
        self.equipped.insert(skill, level);
        if is_selected {
            self.selected.insert(skill);
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
    pub skills: Vec<(Skill, Level)>,
    pub selected: Option<(Skill, Level)>,
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
                    let refl_skills = skills_asset.skills.get_reflected();
                    for skill in &equipped_skills.selected {
                        if let Some(levels) = refl_skills.get(skill) {
                            if let Some(level) = equipped_skills.equipped.get(skill) {
                                if !level.is_last(levels.len()) {
                                    skill_upgrades.push((*skill, level.next()));
                                }
                            }
                        }
                    }
                    if equipped_skills.selected.len() < max_skills.0 as usize {
                        for skill in refl_skills.keys() {
                            if !equipped_skills.equipped.contains_key(skill) {
                                skill_upgrades.push((*skill, Level::default()));
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
    let (Some(entity), Some((sel_skill, sel_level))) = (upgrades.entity, upgrades.selected) else {
        return;
    };
    if let Ok(mut xp_gather_state) = q_xp_gather_state.get_mut(entity) {
        xp_gather_state.upgrade_player_level();
    }
    if let Ok(mut equipped_skills) = q_equipped_skills.get_mut(entity) {
        equipped_skills.set_level(sel_skill, sel_level, true);
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
