use avian3d::{math::*, prelude::*};
use bevy::{
    asset::{AssetLoader, LoadContext, io::Reader},
    platform::collections::HashMap,
    prelude::*,
};
use serde::Deserialize;
use thiserror::Error;

use crate::{
    app::{AppState, InGame},
    camera::MainCameraFocusEvent,
    debug_ui::DebugUi,
    physics::Layer,
    skills::{
        EquippedSkills, HotReloadEquippedSkills, Level, MaxUpgradableSkills, Skill, SkillSpec,
        SkillSpecs, Skills,
    },
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
                    exited: AppState::Menu,
                    entered: AppState::Run,
                },
                spawn_main_player,
            )
            .add_systems(FixedUpdate, move_player.run_if(in_state(AppState::Run)));
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
    pc_handles.meshes = vec![
        meshes.add(
            Capsule3d::new(width, cap_h)
                .mesh()
                .rings(0)
                .latitudes(16)
                .longitudes(32)
                .uv_profile(bevy::render::mesh::CapsuleUvProfile::Aspect)
                .build(),
        ),
    ];
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
    pub speed: f32,
    pub width: f32,
    pub height: f32,
    pub mesh_idx: usize,
    pub material_idx: usize,
    pub max_selected_skills: u8,
    pub default_skills: HashMap<Skill, SkillSpec>,
    pub selected_skills: Vec<Skill>,
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
    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let custom_asset = ron::de::from_bytes::<PlayerCharactersAsset>(&bytes)?;
        Ok(custom_asset)
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
    cmd.queue(SpawnPlayer {
        character: pcs.0.get(0).unwrap().clone(),
        location: Vec2::ZERO,
    });
}

#[derive(Component, Reflect)]
pub struct Player {
    pub speed: f32,
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
        let cap_h = pc.height - 2. * pc.width;

        let Some(skills) = world.get_resource::<Skills>() else {
            return;
        };
        let mut specs = SkillSpecs::default();
        for (skill, spec) in &pc.default_skills {
            specs.0.insert(*skill, (Level::default(), spec.clone()));
        }
        for skill in &pc.selected_skills {
            if let Some(levels) = skills.upgrades.get(skill) {
                specs
                    .0
                    .insert(*skill, (Level::default(), levels[0].clone()));
            }
        }

        world.spawn((
            Name::new(format!("Player {}", pc.name)),
            Player { speed: pc.speed },
            Mesh3d(pc_handles.meshes.get(pc.mesh_idx).unwrap().clone()),
            MeshMaterial3d(pc_handles.materials.get(pc.material_idx).unwrap().clone()),
            Transform::from_xyz(self.location.x, pc.height / 2. + 0.2, self.location.y),
            RigidBody::Kinematic,
            Collider::capsule(pc.width, cap_h),
            CollisionLayers::new([Layer::Player], LayerMask::ALL),
            ShapeCaster::new(
                Collider::capsule(pc.width - 0.05, cap_h - 0.1),
                Vector::ZERO,
                Quaternion::default(),
                Dir3::NEG_Y,
            )
            .with_max_distance(0.11)
            .with_max_hits(1),
            MaxUpgradableSkills(pc.max_selected_skills),
            EquippedSkills::new(&pc.selected_skills),
            specs,
            HotReloadEquippedSkills,
            StateScoped(InGame),
        ));
    }
}

const PLAYER_ACC_STEPS: f32 = 10.;

fn move_player(
    keyboard: Res<ButtonInput<KeyCode>>,
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
            if keyboard.pressed(KeyCode::KeyW) || keyboard.pressed(KeyCode::ArrowUp) {
                vel.y -= acc;
                changed = true;
            }
            if keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft) {
                vel.x -= acc;
                changed = true;
            }
            if keyboard.pressed(KeyCode::KeyS) || keyboard.pressed(KeyCode::ArrowDown) {
                vel.y += acc;
                changed = true;
            }
            if keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight) {
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

        ev_refocus.write(MainCameraFocusEvent {
            focus: player_tr.translation,
        });
    }
}
