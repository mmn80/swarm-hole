use avian3d::prelude::*;
use bevy::prelude::*;

use crate::{app::AppState, physics::Layer};

pub struct TerrainPlugin;

impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Terrain>()
            .init_resource::<Terrain>()
            .add_systems(
                OnTransition {
                    exited: AppState::Menu,
                    entered: AppState::Run,
                },
                setup_terrain,
            )
            .add_systems(OnEnter(AppState::Cleanup), cleanup_terrain);
    }
}

#[derive(Resource, Default, Reflect)]
#[reflect(Resource)]
pub struct Terrain {
    pub ground: Option<Entity>,
}

fn setup_terrain(
    mut terrain: ResMut<Terrain>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut cmd: Commands,
) {
    let ground_size = Vec3::new(1000.0, 1.0, 1000.0);
    let material = materials.add(StandardMaterial {
        base_color: bevy::color::palettes::css::SILVER.into(),
        metallic: 0.0,
        perceptual_roughness: 0.8,
        reflectance: 0.2,
        ..default()
    });

    terrain.ground = Some({
        let id = cmd
            .spawn((
                Mesh3d(meshes.add(Cuboid::new(ground_size.x, ground_size.y, ground_size.z))),
                MeshMaterial3d(material),
                Transform::from_xyz(0.0, -ground_size.y / 2., 0.0),
                RigidBody::Static,
                Collider::cuboid(ground_size.x, ground_size.y, ground_size.z),
                CollisionLayers::new([Layer::Ground], LayerMask::ALL),
            ))
            .id();
        cmd.entity(id)
            .insert(Name::new(format!("Terrain ({id:?})")));
        id
    });

    // let bld_size = Vec3::new(10.0, 4.0, 5.0);
    // let building_id = cmd
    //     .spawn((
    //         PbrBundle {
    //             transform: Transform::from_xyz(0.0, bld_size.y / 2., 10.0),
    //             mesh: meshes.add(Mesh::from(shape::Box::new(
    //                 bld_size.x, bld_size.y, bld_size.z,
    //             ))),
    //             material: materials.building.clone(),
    //             ..default()
    //         },
    //         RigidBody::Static,
    //         Collider::cuboid(bld_size.x, bld_size.y, bld_size.z),
    //         CollisionLayers::new([Layer::Building], ALL_LAYERS),
    //     ))
    //     .id();
    // cmd.entity(building_id)
    //     .insert(Name::new(format!("Building ({building_id:?})")));
}

fn cleanup_terrain(mut terrain: ResMut<Terrain>, mut cmd: Commands) {
    if let Some(entity) = terrain.ground {
        cmd.entity(entity).despawn_recursive();
    }
    terrain.ground = None;
}
