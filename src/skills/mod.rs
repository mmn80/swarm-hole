use bevy::{
    app::PluginGroupBuilder,
    asset::{io::Reader, AssetLoader, AsyncReadExt, LoadContext},
    prelude::*,
    utils::{thiserror, thiserror::Error, BoxedFuture},
};
use serde::Deserialize;

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
            .add_systems(Startup, setup_skills_asset_handle);
    }
}

#[derive(Resource, Default)]
pub struct SkillsAssetHandle(Handle<SkillsAsset>);

fn setup_skills_asset_handle(
    mut skills_asset_handle: ResMut<SkillsAssetHandle>,
    asset_server: Res<AssetServer>,
) {
    skills_asset_handle.0 = asset_server.load("player.skills.ron");
}

#[derive(Clone, Reflect, Debug, Deserialize)]
pub enum Skill {
    Melee(Vec<Melee>),
    Laser(Vec<Laser>),
}

#[derive(Asset, TypePath, Debug, Deserialize)]
pub struct SkillsAsset(pub Vec<Skill>);

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
