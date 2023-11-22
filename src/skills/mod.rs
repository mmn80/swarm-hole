use std::fmt;

use bevy::{
    app::PluginGroupBuilder,
    asset::{io::Reader, AssetLoader, AsyncReadExt, LoadContext},
    prelude::*,
    utils::{thiserror, thiserror::Error, BoxedFuture, HashMap, HashSet},
};
use rand::prelude::*;
use serde::Deserialize;

use crate::app::AppState;

use self::{
    health::HealthPlugin,
    laser::LaserPlugin,
    melee::MeleePlugin,
    xp::{XpGatherState, XpPlugin},
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
            .register_type::<Attribute>()
            .register_type::<Value>()
            .init_asset::<SkillsAsset>()
            .init_asset_loader::<SkillsAssetLoader>()
            .init_resource::<Skills>()
            .init_resource::<SkillUpgradeOptions>()
            .add_systems(Startup, setup_skills_asset_handle)
            .add_systems(Update, skills_asset_on_load)
            .add_systems(Update, init_upgrade_menu.run_if(in_state(AppState::Run)))
            .add_systems(
                Update,
                apply_upgrade_selection.run_if(in_state(AppState::Upgrade)),
            );
    }
}

#[derive(Copy, Clone, Reflect, Debug, Deserialize)]
pub enum Value {
    F(f32),
    U(u32),
    Af(f32),
    Au(u32),
    Mf(f32),
}

impl Value {
    pub fn as_f32(&self) -> f32 {
        match self {
            Value::F(v) => *v,
            Value::U(v) => *v as f32,
            Value::Af(v) => *v,
            Value::Au(v) => *v as f32,
            Value::Mf(v) => *v,
        }
    }

    pub fn delta(&self, other: Value) -> Option<Value> {
        match self {
            Value::F(v1) => {
                if let Value::F(v2) = other {
                    Some(Value::F(v1 - v2))
                } else {
                    None
                }
            }
            Value::U(v1) => {
                if let Value::U(v2) = other {
                    Some(Value::U(v1 - v2))
                } else {
                    None
                }
            }
            Value::Af(v1) => {
                if let Value::Af(v2) = other {
                    Some(Value::Af(v1 - v2))
                } else {
                    None
                }
            }
            Value::Au(v1) => {
                if let Value::Au(v2) = other {
                    Some(Value::Au(v1 - v2))
                } else {
                    None
                }
            }
            Value::Mf(v1) => {
                if let Value::Mf(v2) = other {
                    Some(Value::Mf(v1 - v2))
                } else {
                    None
                }
            }
        }
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Copy, Clone, Reflect, Debug, Deserialize, Hash)]
pub enum Attribute {
    MaxHp,
    HpPerSec,
    Range,
    Acceleration,
    Dps,
    Duration,
    Cooldown,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Copy, Clone, Reflect, Debug, Deserialize, Hash)]
pub enum Skill {
    Health,
    HealthRegen,
    XpGather,
    Melee,
    Laser,
}

pub trait IsSkill {
    fn skill() -> Skill;
}

// skill component initialization

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Default, Debug, Reflect, Deserialize)]
pub struct Level(u8);

impl Level {
    pub fn is_first(&self) -> bool {
        self.0 == 0
    }

    pub fn next(&self, levels_count: usize) -> Option<Self> {
        if self.0 as usize >= levels_count - 1 {
            None
        } else {
            Some(Self(self.0 + 1))
        }
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

pub type SkillSpec = HashMap<Attribute, Value>;

#[derive(Component, Default)]
pub struct SkillSpecs(pub HashMap<Skill, (Level, SkillSpec)>);

#[derive(Component, Clone, Default)]
pub struct EquippedSkills {
    equipped: HashMap<Skill, Level>,
    selected: HashSet<Skill>,
}

impl EquippedSkills {
    pub fn is_equipped(&self, skill: Skill) -> bool {
        self.equipped.contains_key(&skill)
    }

    fn set_level(&mut self, skill: Skill, level: Level, is_selected: bool) {
        self.equipped.insert(skill, level);
        if is_selected {
            self.selected.insert(skill);
        }
    }
}

pub fn apply_skill_specs<T: Component + Struct + Default + IsSkill>(
    skills_meta: Res<Skills>,
    q_no_skill: Query<(Entity, &SkillSpecs), Without<T>>,
    mut q_skill: Query<(Entity, &mut SkillSpecs, &mut T, &mut EquippedSkills)>,
    mut cmd: Commands,
) {
    let skill = T::skill();
    for (entity, specs) in &q_no_skill {
        if specs.0.contains_key(&skill) {
            cmd.entity(entity).insert(T::default());
        }
    }
    for (entity, mut specs, mut refl_struct, mut equipped) in &mut q_skill {
        if let Some((level, spec)) = specs.0.get(&skill) {
            for (attr, val) in spec {
                if let Some(attr_meta) = skills_meta.attributes.get(attr) {
                    if let Some(fld) = refl_struct.field_mut(&attr_meta.field_name) {
                        match val {
                            Value::F(v) => fld.downcast_mut::<f32>().map(|f| *f = *v),
                            Value::U(v) => fld.downcast_mut::<u32>().map(|f| *f = *v),
                            Value::Af(v) => fld.downcast_mut::<f32>().map(|f| *f += *v),
                            Value::Au(v) => fld.downcast_mut::<u32>().map(|f| *f += *v),
                            Value::Mf(v) => fld.downcast_mut::<f32>().map(|f| *f *= *v / 100.),
                        };
                    } else {
                        error!(
                            "Field {} not found for skill {skill:?}!",
                            attr_meta.field_name
                        );
                    }
                } else {
                    error!("Attribute {attr:?} not found for skill {skill:?}!");
                }
            }

            equipped.set_level(skill, *level, false);

            specs.0.remove(&skill);
            if specs.0.is_empty() {
                cmd.entity(entity).remove::<SkillSpecs>();
            }
        }
    }
}

// skill upgrades

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
    skills: Res<Skills>,
    q_xp_gather_state: Query<(Entity, &XpGatherState, &MaxUpgradableSkills)>,
    q_equipped_skills: Query<&EquippedSkills>,
) {
    for (entity, xp_gather_state, max_skills) in &q_xp_gather_state {
        if xp_gather_state.get_gather_level() > xp_gather_state.get_player_level() {
            if let Ok(equipped) = q_equipped_skills.get(entity) {
                let mut skill_upgrades = vec![];
                for skill in &equipped.selected {
                    if let Some(levels) = skills.upgrades.get(skill) {
                        if let Some(level) = equipped.equipped.get(skill) {
                            if let Some(next_level) = level.next(levels.len()) {
                                skill_upgrades.push((*skill, next_level));
                            }
                        }
                    }
                }
                if equipped.selected.len() < max_skills.0 as usize {
                    for skill in skills.upgrades.keys() {
                        if !equipped.selected.contains(skill) {
                            skill_upgrades.push((*skill, Level::default()));
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
    skills: Res<Skills>,
    mut q_xp_gather_state: Query<&mut XpGatherState>,
    mut q_equipped_skills: Query<&mut EquippedSkills>,
    mut cmd: Commands,
) {
    let (Some(entity), Some((skill, level))) = (upgrades.entity, upgrades.selected) else {
        return;
    };
    if let Ok(mut xp_gather_state) = q_xp_gather_state.get_mut(entity) {
        xp_gather_state.upgrade_player_level();
    }
    if let Ok(mut equipped_skills) = q_equipped_skills.get_mut(entity) {
        equipped_skills.set_level(skill, level, true);
    }
    if let Some(levels) = skills.upgrades.get(&skill) {
        if let Some(spec) = level.index(levels) {
            cmd.entity(entity)
                .insert(SkillSpecs(HashMap::from([(skill, (level, spec.clone()))])));
        } else {
            error!("Did not find level {level} upgrades for equipped skill {skill:?}.");
        }
    } else {
        error!("Did not find upgrades for equipped skill {skill:?}.");
    }
    upgrades.entity = None;
    upgrades.skills.clear();
    upgrades.selected = None;
    next_state.set(AppState::Run);
}

// asset loading

#[derive(Asset, TypePath, Debug, Deserialize)]
pub struct SkillsAsset {
    pub skills: HashMap<Skill, String>,
    pub attributes: HashMap<Attribute, AttributeMeta>,
    pub upgrades: HashMap<Skill, Vec<SkillSpec>>,
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

#[derive(Clone, Debug, Deserialize)]
pub struct AttributeMeta {
    pub field_name: String,
    pub ui_name: String,
}

#[derive(Resource, Default)]
pub struct Skills {
    pub handle: Handle<SkillsAsset>,
    pub attributes: HashMap<Attribute, AttributeMeta>,
    pub attributes_inv: HashMap<String, Attribute>,
    pub skills: HashMap<Skill, String>,
    pub upgrades: HashMap<Skill, Vec<SkillSpec>>,
}

fn setup_skills_asset_handle(mut skills_meta: ResMut<Skills>, asset_server: Res<AssetServer>) {
    skills_meta.handle = asset_server.load("all.skills.ron");
}

#[derive(Component)]
pub struct HotReloadEquippedSkills;

fn skills_asset_on_load(
    mut skills: ResMut<Skills>,
    mut skills_asset_events: EventReader<AssetEvent<SkillsAsset>>,
    skills_assets: Res<Assets<SkillsAsset>>,
    q_equipped: Query<(Entity, &EquippedSkills), With<HotReloadEquippedSkills>>,
    mut cmd: Commands,
) {
    for ev in skills_asset_events.read() {
        let h = skills.handle.clone();
        if ev.is_loaded_with_dependencies(&h) {
            if let Some(asset) = skills_assets.get(h) {
                // hot reload skills meta
                skills.skills = asset.skills.clone();
                skills.attributes = asset.attributes.clone();
                skills.attributes_inv.clear();
                for (attr, attr_mata) in &asset.attributes {
                    skills
                        .attributes_inv
                        .insert(attr_mata.field_name.clone(), *attr);
                }
                skills.upgrades = asset.upgrades.clone();

                // hot reload skill components
                for (entity, equipped) in &q_equipped {
                    let mut specs = SkillSpecs(HashMap::new());
                    for (skill, level) in &equipped.equipped {
                        if let Some(levels) = skills.upgrades.get(skill) {
                            if let Some(spec) = level.index(levels) {
                                specs.0.insert(*skill, (*level, spec.clone()));
                            } else {
                                error!("Hot reload: did not find level {level} upgrades for equipped skill {skill:?}.");
                            }
                        } else {
                            error!(
                                "Hot reload: did not find upgrades for equipped skill {skill:?}."
                            );
                        }
                    }
                    if !specs.0.is_empty() {
                        cmd.entity(entity).insert(specs);
                    }
                }
            }
        }
    }
}
