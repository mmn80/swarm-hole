use bevy::{
    asset::{io::Reader, AssetLoader, AsyncReadExt, LoadContext},
    ecs::world::Command,
    prelude::*,
    utils::HashMap,
};
use bevy_xpbd_3d::prelude::*;
use rand::{distributions::WeightedIndex, prelude::*};
use serde::Deserialize;
use thiserror::Error;

use crate::{
    app::{AppState, RunState},
    physics::{Layer, ALL_LAYERS},
    skills::{EquippedSkills, Level, Skill, SkillSpec, SkillSpecs},
};

pub struct NpcPlugin;

impl Plugin for NpcPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<NonPlayerCharacter>()
            .init_asset::<NonPlayerCharactersAsset>()
            .init_asset_loader::<NonPlayerCharactersAssetLoader>()
            .init_resource::<NpcHandles>()
            .add_systems(Startup, setup_npc_handles)
            .add_systems(
                OnTransition {
                    exited: AppState::Menu,
                    entered: AppState::Run,
                },
                spawn_start_npcs,
            )
            .add_systems(Update, hot_reload_npcs)
            .add_systems(OnEnter(AppState::Cleanup), cleanup_npcs);
    }
}

#[derive(Resource, Default)]
pub struct NpcHandles {
    pub meshes: Vec<Handle<Mesh>>,
    pub materials: Vec<Handle<StandardMaterial>>,
    pub config: Handle<NonPlayerCharactersAsset>,
}

fn setup_npc_handles(
    mut npc_handles: ResMut<NpcHandles>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    npc_handles.config = asset_server.load("all.npcs.ron");
    npc_handles.meshes = vec![
        meshes.add(Sphere::new(0.5).mesh().ico(5).unwrap()),
        meshes.add(Sphere::new(1.).mesh().ico(8).unwrap()),
    ];
    npc_handles.materials = vec![
        materials.add(StandardMaterial {
            base_color: bevy::color::palettes::css::LIMEGREEN.into(),
            metallic: 0.8,
            perceptual_roughness: 0.3,
            ..default()
        }),
        materials.add(StandardMaterial {
            base_color: bevy::color::palettes::css::TOMATO.into(),
            metallic: 0.8,
            perceptual_roughness: 0.3,
            ..default()
        }),
    ];
}

#[derive(Reflect, Clone, Debug, Deserialize)]
pub struct NonPlayerCharacter {
    pub name: String,
    pub xp_drop: u32,
    pub frequency: f32,
    pub radius: f32,
    pub mesh_idx: usize,
    pub material_idx: usize,
    pub skills: HashMap<Skill, SkillSpec>,
}

#[derive(Asset, TypePath, Debug, Deserialize)]
pub struct NonPlayerCharactersAsset(pub Vec<NonPlayerCharacter>);

impl NonPlayerCharactersAsset {
    pub fn get_npc_by_index(&self, index: NpcAssetIndex) -> Option<&NonPlayerCharacter> {
        self.0.get(index.0)
    }
}

#[derive(Default)]
pub struct NonPlayerCharactersAssetLoader;

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum NonPlayerCharactersAssetLoaderError {
    #[error("Could not load asset: {0}")]
    Io(#[from] std::io::Error),
    #[error("Could not parse RON: {0}")]
    RonSpannedError(#[from] ron::error::SpannedError),
}

impl AssetLoader for NonPlayerCharactersAssetLoader {
    type Asset = NonPlayerCharactersAsset;
    type Settings = ();
    type Error = NonPlayerCharactersAssetLoaderError;
    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let custom_asset = ron::de::from_bytes::<NonPlayerCharactersAsset>(&bytes)?;
        Ok(custom_asset)
    }

    fn extensions(&self) -> &[&str] {
        &["npcs.ron"]
    }
}

#[derive(Component, Reflect, Clone)]
pub struct Npc {
    pub xp_drop: u32,
}

pub struct SpawnNpc {
    pub character: NonPlayerCharacter,
    pub npc_index: NpcAssetIndex,
    pub location: Vec2,
}

impl Command for SpawnNpc {
    fn apply(self, world: &mut World) {
        {
            let Some(npc_handles) = world.get_resource::<NpcHandles>() else {
                return;
            };
            let npc = &self.character;

            let mut specs = SkillSpecs::default();
            for (skill, spec) in &npc.skills {
                specs.0.insert(*skill, (Level::default(), spec.clone()));
            }

            let id = world
                .spawn((
                    Npc {
                        xp_drop: npc.xp_drop,
                    },
                    HotReloadNpc(self.npc_index),
                    Mesh3d(npc_handles.meshes.get(npc.mesh_idx).unwrap().clone()),
                    MeshMaterial3d(npc_handles.materials.get(npc.material_idx).unwrap().clone()),
                    Transform::from_xyz(self.location.x, npc.radius + 0.02, self.location.y),
                    RigidBody::Kinematic,
                    Collider::sphere(npc.radius),
                    CollisionLayers::new([Layer::NPC], ALL_LAYERS),
                    EquippedSkills::default(),
                    specs,
                ))
                .id();
            world
                .entity_mut(id)
                .insert(Name::new(format!("NPC {:?} ({id:?})", npc.name)));
        }

        if let Some(mut run_state) = world.get_resource_mut::<RunState>() {
            run_state.live_npcs += 1;
        };
    }
}

fn spawn_start_npcs(mut cmd: Commands) {
    cmd.queue(SpawnRandomNpcs {
        count: 5000,
        distance: 20.,
    });
}

pub struct SpawnRandomNpcs {
    pub count: usize,
    pub distance: f32,
}

impl Command for SpawnRandomNpcs {
    fn apply(self, world: &mut World) {
        info!("spawning {} NPCs...", self.count);
        let npcs = {
            let Some(npc_handles) = world.get_resource::<NpcHandles>() else {
                error!("NPC handles resource not found!");
                return;
            };
            let Some(npc_assets) = world.get_resource::<Assets<NonPlayerCharactersAsset>>() else {
                error!("NPC assets not found!");
                return;
            };
            let Some(npcs) = npc_assets.get(&npc_handles.config) else {
                error!("NPC config asset not loaded!");
                return;
            };
            npcs.0.clone()
        };

        let mut rng = thread_rng();
        let npc_idx = WeightedIndex::new(npcs.iter().map(|npc| npc.frequency)).unwrap();

        let w = ((self.count as f32).sqrt() / 2.).ceil() as i32;
        let dist = (self.distance - 4.) / 2.;
        let mut n = 0;
        for xi in -w..=w {
            for zi in -w..=w {
                let idx = npc_idx.sample(&mut rng);
                let npc = &npcs[idx];
                let x = xi as f32 * self.distance + rng.gen_range(-dist..dist);
                let z = zi as f32 * self.distance + rng.gen_range(-dist..dist);

                SpawnNpc {
                    character: npc.clone(),
                    npc_index: NpcAssetIndex(idx),
                    location: Vec2::new(x, z),
                }
                .apply(world);

                n += 1;
                if n == self.count {
                    break;
                }
            }
            if n == self.count {
                break;
            }
        }
    }
}

#[derive(PartialEq, Copy, Clone)]
pub struct NpcAssetIndex(usize);

#[derive(Component)]
pub struct HotReloadNpc(NpcAssetIndex);

fn hot_reload_npcs(
    npc_handles: Res<NpcHandles>,
    npcs_assets: Res<Assets<NonPlayerCharactersAsset>>,
    mut skills_asset_events: EventReader<AssetEvent<NonPlayerCharactersAsset>>,
    mut q_npcs: Query<(Entity, &mut Npc, &HotReloadNpc)>,
    mut cmd: Commands,
) {
    for ev in skills_asset_events.read() {
        let h = npc_handles.config.clone();
        if ev.is_loaded_with_dependencies(&h) {
            if let Some(asset) = npcs_assets.get(&h) {
                for (entity, mut npc, hot_reload_npc) in &mut q_npcs {
                    if let Some(npc_src) = asset.get_npc_by_index(hot_reload_npc.0) {
                        npc.xp_drop = npc_src.xp_drop;
                        let mut specs = SkillSpecs::default();
                        for (skill, spec) in &npc_src.skills {
                            specs.0.insert(*skill, (Level::default(), spec.clone()));
                        }
                        cmd.entity(entity).insert(specs);
                    }
                }
            }
        }
    }
}

fn cleanup_npcs(q_npc: Query<Entity, With<Npc>>, mut cmd: Commands) {
    for entity in &q_npc {
        cmd.entity(entity).despawn_recursive();
    }
}
