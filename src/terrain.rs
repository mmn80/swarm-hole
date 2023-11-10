use bevy::prelude::*;
use bevy_xpbd_3d::prelude::*;

use crate::{
    materials::BasicMaterials,
    physics::{Layer, ALL_LAYERS},
};

pub struct TerrainPlugin;

impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Terrain>()
            .init_resource::<Terrain>()
            .add_systems(Startup, setup_terrain);
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
    materials: Res<BasicMaterials>,
    mut cmd: Commands,
) {
    let ground_size = Vec3::new(1000.0, 1.0, 1000.0);

    terrain.ground = Some({
        let id = cmd
            .spawn((
                PbrBundle {
                    transform: Transform::from_xyz(0.0, -ground_size.y / 2., 0.0),
                    mesh: meshes.add(Mesh::from(shape::Box::new(
                        ground_size.x,
                        ground_size.y,
                        ground_size.z,
                    ))),
                    material: materials.terrain.clone(),
                    ..default()
                },
                RigidBody::Static,
                Collider::cuboid(ground_size.x, ground_size.y, ground_size.z),
                CollisionLayers::new([Layer::Ground], ALL_LAYERS),
            ))
            .id();
        cmd.entity(id)
            .insert(Name::new(format!("Terrain ({id:?})")));
        id
    });

    let bld_size = Vec3::new(10.0, 4.0, 5.0);
    let building_id = cmd
        .spawn((
            PbrBundle {
                transform: Transform::from_xyz(0.0, bld_size.y / 2., 10.0),
                mesh: meshes.add(Mesh::from(shape::Box::new(
                    bld_size.x, bld_size.y, bld_size.z,
                ))),
                material: materials.building.clone(),
                ..default()
            },
            RigidBody::Static,
            Collider::cuboid(bld_size.x, bld_size.y, bld_size.z),
            CollisionLayers::new([Layer::Building], ALL_LAYERS),
        ))
        .id();
    cmd.entity(building_id)
        .insert(Name::new(format!("Building ({building_id:?})")));
}
