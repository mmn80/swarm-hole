use bevy::{
    asset::{io::Reader, AssetLoader, AsyncReadExt, LoadContext},
    ecs::system::Command,
    prelude::*,
    utils::{thiserror, thiserror::Error, BoxedFuture},
};
use bevy_xpbd_3d::{math::*, prelude::*, PhysicsSchedule, PhysicsStepSet};
use serde::Deserialize;

use crate::{
    app::AppState,
    camera::MainCameraFocusEvent,
    debug_ui::DebugUi,
    physics::{Layer, ALL_LAYERS},
    skills::{health::Health, HotReloadEquippedSkills, Skill, SkillsAsset, SkillsAssetHandle},
};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<PlayerCharacter>()
            .register_type::<Player>()
            .init_asset::<PlayerCharactersAsset>()
            .init_asset_loader::<PlayerCharactersAssetLoader>()
            .init_resource::<PcHandles>()
            .add_systems(Startup, setup_player_handles)
            .add_systems(
                OnTransition {
                    from: AppState::Menu,
                    to: AppState::Run,
                },
                spawn_main_player,
            )
            .add_systems(
                PhysicsSchedule,
                move_player
                    .before(PhysicsStepSet::BroadPhase)
                    .run_if(in_state(AppState::Run)),
            )
            .add_systems(OnEnter(AppState::Cleanup), cleanup_players);
    }
}

#[derive(Resource, Default)]
pub struct PcHandles {
    pub meshes: Vec<Handle<Mesh>>,
    pub materials: Vec<Handle<StandardMaterial>>,
    pub config: Handle<PlayerCharactersAsset>,
}

fn setup_player_handles(
    mut pc_handles: ResMut<PcHandles>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    pc_handles.config = asset_server.load("all.pcs.ron");

    let (height, width) = (2., 0.3);
    let cap_h = height - 2. * width;
    pc_handles.meshes = vec![meshes.add(Mesh::from(shape::Capsule {
        radius: width,
        rings: 0,
        depth: cap_h,
        latitudes: 16,
        longitudes: 32,
        uv_profile: shape::CapsuleUvProfile::Aspect,
    }))];
    pc_handles.materials = vec![materials.add(StandardMaterial {
        base_color: Color::BLACK,
        metallic: 0.0,
        perceptual_roughness: 0.5,
        ..default()
    })];
}

#[derive(Reflect, Clone, Debug, Deserialize)]
pub struct PlayerCharacter {
    pub name: String,
    pub max_hp: u32,
    pub hp_regen_per_sec: f32,
    pub speed: f32,
    pub width: f32,
    pub height: f32,
    pub mesh_idx: usize,
    pub material_idx: usize,
    pub starting_skills: Vec<Skill>,
}

#[derive(Asset, TypePath, Debug, Deserialize)]
pub struct PlayerCharactersAsset(pub Vec<PlayerCharacter>);

#[derive(Default)]
pub struct PlayerCharactersAssetLoader;

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum PlayerCharactersAssetLoaderError {
    #[error("Could not load asset: {0}")]
    Io(#[from] std::io::Error),
    #[error("Could not parse RON: {0}")]
    RonSpannedError(#[from] ron::error::SpannedError),
}

impl AssetLoader for PlayerCharactersAssetLoader {
    type Asset = PlayerCharactersAsset;
    type Settings = ();
    type Error = PlayerCharactersAssetLoaderError;
    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        _settings: &'a (),
        _load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;
            let custom_asset = ron::de::from_bytes::<PlayerCharactersAsset>(&bytes)?;
            Ok(custom_asset)
        })
    }

    fn extensions(&self) -> &[&str] {
        &["pcs.ron"]
    }
}

fn spawn_main_player(
    pc_handles: Res<PcHandles>,
    pc_assets: Res<Assets<PlayerCharactersAsset>>,
    mut cmd: Commands,
) {
    let Some(pcs) = pc_assets.get(&pc_handles.config) else {
        error!("PC config asset not loaded!");
        return;
    };
    cmd.add(SpawnPlayer {
        character: pcs.0.get(0).unwrap().clone(),
        location: Vec2::ZERO,
    });
}

#[derive(Component, Reflect)]
pub struct Player {
    pub speed: f32,
    pub max_hp: u32,
    pub hp_regen_per_sec: f32,
}

pub struct SpawnPlayer {
    pub character: PlayerCharacter,
    pub location: Vec2,
}

impl Command for SpawnPlayer {
    fn apply(self, world: &mut World) {
        let pc = &self.character;
        let Some(pc_handles) = world.get_resource::<PcHandles>() else {
            return;
        };
        let skills = {
            let Some(skills_assets) = world.get_resource::<Assets<SkillsAsset>>() else {
                return;
            };
            let Some(skills_asset_handle) = world.get_resource::<SkillsAssetHandle>() else {
                return;
            };
            let Some(skills_asset) = skills_assets.get(skills_asset_handle.0.clone()) else {
                return;
            };
            let mut skills = vec![];
            for skill in &skills_asset.0 {
                for starting_skill in &pc.starting_skills {
                    if skill.same_skill(starting_skill) {
                        skills.push(skill.clone())
                    }
                }
            }
            skills
        };

        let cap_h = pc.height - 2. * pc.width;
        let id = world
            .spawn((
                Player {
                    speed: pc.speed,
                    max_hp: pc.max_hp,
                    hp_regen_per_sec: pc.hp_regen_per_sec,
                },
                Health(pc.max_hp as f32),
                PbrBundle {
                    transform: Transform::from_xyz(
                        self.location.x,
                        pc.height / 2. + 0.2,
                        self.location.y,
                    ),
                    mesh: pc_handles.meshes.get(pc.mesh_idx).unwrap().clone(),
                    material: pc_handles.materials.get(pc.material_idx).unwrap().clone(),
                    ..default()
                },
                RigidBody::Kinematic,
                Collider::capsule(cap_h, pc.width),
                CollisionLayers::new([Layer::Player], ALL_LAYERS),
                ShapeCaster::new(
                    Collider::capsule(cap_h - 0.1, pc.width - 0.05),
                    Vector::ZERO,
                    Quaternion::default(),
                    Vector::NEG_Y,
                )
                .with_max_time_of_impact(0.11)
                .with_max_hits(1),
                HotReloadEquippedSkills,
            ))
            .id();
        world
            .entity_mut(id)
            .insert(Name::new(format!("Player {} ({id:?})", pc.name)));

        Skill::insert_components(&skills, 0, id, world);
    }
}

const PLAYER_ACC_STEPS: f32 = 10.;

fn move_player(
    keyboard: Res<Input<KeyCode>>,
    debug_ui: Res<DebugUi>,
    mut q_player: Query<(&Transform, &Player, &mut LinearVelocity, &ShapeHits)>,
    mut ev_refocus: EventWriter<MainCameraFocusEvent>,
) {
    for (player_tr, player, mut linear_velocity, ground_hits) in &mut q_player {
        if !ground_hits.is_empty() {
            linear_velocity.y = 0.0;
        } else {
            linear_velocity.y -= 0.4;
        }

        let acc = player.speed / PLAYER_ACC_STEPS;
        let mut vel = Vec2::new(linear_velocity.x, linear_velocity.z);
        if !debug_ui.has_focus() {
            let mut changed = false;
            if keyboard.pressed(KeyCode::W) || keyboard.pressed(KeyCode::Up) {
                vel.y -= acc;
                changed = true;
            }
            if keyboard.pressed(KeyCode::A) || keyboard.pressed(KeyCode::Left) {
                vel.x -= acc;
                changed = true;
            }
            if keyboard.pressed(KeyCode::S) || keyboard.pressed(KeyCode::Down) {
                vel.y += acc;
                changed = true;
            }
            if keyboard.pressed(KeyCode::D) || keyboard.pressed(KeyCode::Right) {
                vel.x += acc;
                changed = true;
            }
            if !changed {
                vel *= 0.8;
            }
            if keyboard.just_pressed(KeyCode::Space) && !ground_hits.is_empty() {
                linear_velocity.y += 20.0;
            }
        }
        vel = vel.clamp_length_max(player.speed);

        linear_velocity.x = vel.x;
        linear_velocity.z = vel.y;
        linear_velocity.y *= 0.98;

        ev_refocus.send(MainCameraFocusEvent {
            focus: player_tr.translation,
        });
    }
}

fn cleanup_players(q_player: Query<Entity, With<Player>>, mut cmd: Commands) {
    for entity in &q_player {
        cmd.entity(entity).despawn_recursive();
    }
}
