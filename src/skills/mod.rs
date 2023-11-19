use bevy::{
    app::PluginGroupBuilder,
    asset::{io::Reader, AssetLoader, AsyncReadExt, LoadContext},
    ecs::system::Command,
    prelude::*,
    utils::{thiserror, thiserror::Error, BoxedFuture},
};
use serde::Deserialize;

use self::{
    health::HealthPlugin,
    laser::{Laser, LaserPlugin},
    melee::{Melee, MeleePlugin},
    xp_drops::{XpDropsPlugin, XpGather},
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
        app.register_type::<Skill>()
            .init_asset::<SkillsAsset>()
            .init_asset_loader::<SkillsAssetLoader>()
            .init_resource::<SkillsAssetHandle>()
            .add_systems(Startup, setup_skills_asset_handle)
            .add_systems(Update, hot_reload_equipped_skills);
    }
}

#[derive(Resource, Default)]
pub struct SkillsAssetHandle(pub Handle<SkillsAsset>);

fn setup_skills_asset_handle(
    mut skills_asset_handle: ResMut<SkillsAssetHandle>,
    asset_server: Res<AssetServer>,
) {
    skills_asset_handle.0 = asset_server.load("all.skills.ron");
}

#[derive(PartialEq, Eq, Copy, Clone)]
pub struct SkillIndex(u8);

#[derive(Clone, Reflect, Debug, Deserialize)]
pub enum Skill {
    XpGather(Vec<XpGather>),
    Melee(Vec<Melee>),
    Laser(Vec<Laser>),
}

impl Skill {
    pub fn get_index(&self) -> SkillIndex {
        match self {
            Skill::XpGather(_) => SkillIndex(0),
            Skill::Melee(_) => SkillIndex(1),
            Skill::Laser(_) => SkillIndex(2),
        }
    }

    pub fn same_skill(&self, other: &Skill) -> bool {
        self.get_index() == other.get_index()
    }

    pub fn insert_component(&self, level: u8, entity: Entity, world: &mut World) {
        let Some(mut ent) = world.get_entity_mut(entity) else {
            return;
        };
        let lvl = level as usize;
        match self {
            Skill::XpGather(levels) => {
                ent.insert(levels[lvl].clone());
            }
            Skill::Melee(levels) => {
                ent.insert(levels[lvl].clone());
            }
            Skill::Laser(levels) => {
                ent.insert(levels[lvl].clone());
            }
        }
        let skill = self.get_index();
        if let Some(mut equipped) = ent.get_mut::<EquippedSkills>() {
            equipped.update_skill(skill, level);
        } else {
            ent.insert(EquippedSkills(vec![EquippedSkill { skill, level }]));
        }
    }

    pub fn insert_components(skills: &Vec<Skill>, level: u8, entity: Entity, world: &mut World) {
        for skill in skills {
            skill.insert_component(level, entity, world);
        }
    }
}

pub struct UpdateSkillComponent {
    pub entity: Entity,
    pub skill: Skill,
    pub level: u8,
}

impl Command for UpdateSkillComponent {
    fn apply(self, world: &mut World) {
        self.skill.insert_component(self.level, self.entity, world);
    }
}

pub struct EquippedSkill {
    pub skill: SkillIndex,
    pub level: u8,
}

#[derive(Component)]
pub struct EquippedSkills(pub Vec<EquippedSkill>);

impl EquippedSkills {
    pub fn update_skill(&mut self, skill: SkillIndex, level: u8) {
        let mut found = false;
        for equipped in &mut self.0 {
            if equipped.skill == skill {
                equipped.level = level;
                found = true;
                break;
            }
        }
        if !found {
            self.0.push(EquippedSkill { skill, level });
        }
    }
}

#[derive(Component)]
pub struct HotReloadEquippedSkills;

fn hot_reload_equipped_skills(
    skills_asset_handle: Res<SkillsAssetHandle>,
    skills_asset: Res<Assets<SkillsAsset>>,
    mut skills_asset_events: EventReader<AssetEvent<SkillsAsset>>,
    q_equipped: Query<(Entity, &EquippedSkills), With<HotReloadEquippedSkills>>,
    mut cmd: Commands,
) {
    for ev in skills_asset_events.read() {
        let h = skills_asset_handle.0.clone();
        if ev.is_loaded_with_dependencies(&h) {
            if let Some(skills) = skills_asset.get(h) {
                for (entity, equipped_skills) in &q_equipped {
                    for equipped_skill in &equipped_skills.0 {
                        if let Some(skill) = skills.get_skill_by_index(equipped_skill.skill) {
                            cmd.add(UpdateSkillComponent {
                                entity,
                                skill: skill.clone(),
                                level: equipped_skill.level,
                            });
                        }
                    }
                }
            }
        }
    }
}

#[derive(Asset, TypePath, Debug, Deserialize)]
pub struct SkillsAsset(pub Vec<Skill>);

impl SkillsAsset {
    pub fn get_skill_by_index(&self, skill: SkillIndex) -> Option<&Skill> {
        self.0.iter().find(|s| s.get_index() == skill)
    }
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
