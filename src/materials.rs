use bevy::prelude::*;

pub struct BasicMaterialsPlugin;

impl Plugin for BasicMaterialsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BasicMaterials>();
    }
}

#[derive(Resource, Reflect)]
pub struct BasicMaterials {
    pub terrain: Handle<StandardMaterial>,
    pub player: Handle<StandardMaterial>,
    pub building: Handle<StandardMaterial>,
}

impl FromWorld for BasicMaterials {
    fn from_world(world: &mut World) -> Self {
        let mut materials = world
            .get_resource_mut::<Assets<StandardMaterial>>()
            .unwrap();

        BasicMaterials {
            terrain: materials.add(StandardMaterial {
                base_color: Color::SILVER,
                metallic: 0.0,
                perceptual_roughness: 0.8,
                reflectance: 0.2,
                ..default()
            }),
            player: materials.add(StandardMaterial {
                base_color: Color::BLACK,
                metallic: 0.0,
                perceptual_roughness: 0.5,
                ..default()
            }),
            building: materials.add(StandardMaterial {
                base_color: Color::CRIMSON,
                metallic: 0.2,
                perceptual_roughness: 0.7,
                reflectance: 0.2,
                ..default()
            }),
        }
    }
}
