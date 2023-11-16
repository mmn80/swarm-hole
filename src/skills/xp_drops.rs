use bevy::prelude::*;
use bevy_xpbd_3d::prelude::*;

use crate::{app::AppState, physics::Layer, player::PlayerCharacter};

pub struct XpDropsPlugin;

impl Plugin for XpDropsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<XpDrops>()
            .add_systems(Startup, setup_xp_drops)
            .add_systems(
                Update,
                (gather_xp, slow_xp_drops).run_if(in_state(AppState::Run)),
            )
            .add_systems(OnEnter(AppState::Cleanup), cleanup_xp_drops);
    }
}

#[derive(Resource, Default)]
pub struct XpDrops {
    pub xp_drop_small: Handle<StandardMaterial>,
    pub xp_drop_big: Handle<StandardMaterial>,
}

fn setup_xp_drops(mut xp_drops: ResMut<XpDrops>, mut materials: ResMut<Assets<StandardMaterial>>) {
    xp_drops.xp_drop_small = materials.add(StandardMaterial {
        base_color: Color::rgb(1.0, 4.0, 1.0),
        metallic: 0.8,
        perceptual_roughness: 0.4,
        reflectance: 0.9,
        ..default()
    });
    xp_drops.xp_drop_big = materials.add(StandardMaterial {
        base_color: Color::rgb(4.0, 1.0, 1.0),
        metallic: 0.8,
        perceptual_roughness: 0.4,
        reflectance: 0.9,
        ..default()
    });
}

#[derive(Component)]
pub struct XpDrop(pub u32);

impl XpDrop {
    pub fn is_big(drop: u32) -> bool {
        drop > 5
    }

    pub fn get_height(drop: u32) -> f32 {
        if XpDrop::is_big(drop) {
            0.4
        } else {
            0.2
        }
    }
}

#[derive(Component)]
pub struct XpGather(pub u32);

fn gather_xp(
    time: Res<Time>,
    q_space: SpatialQuery,
    mut q_xp_gather: Query<(&Transform, &PlayerCharacter, &mut XpGather)>,
    mut q_xp_drop: Query<(Entity, &Transform, &mut LinearVelocity, &XpDrop)>,
    mut cmd: Commands,
) {
    for (tr_player, player_character, mut xp_gather) in &mut q_xp_gather {
        for ent in q_space
            .shape_intersections(
                &Collider::ball(player_character.gather_range),
                tr_player.translation,
                Quat::default(),
                SpatialQueryFilter::new().with_masks([Layer::Building]),
            )
            .iter()
        {
            if let Ok((ent, tr_xp, mut lin_vel, xp_drop)) = q_xp_drop.get_mut(*ent) {
                let mut delta = tr_player.translation - tr_xp.translation;
                if delta.length() < XpDrop::get_height(xp_drop.0) + 1. {
                    xp_gather.0 += xp_drop.0;
                    cmd.entity(ent).despawn_recursive();
                } else {
                    lin_vel.y = 0.;
                    let old_speed = lin_vel.length();
                    delta.y = 0.;
                    delta = delta.normalize()
                        * (old_speed + time.delta_seconds() * player_character.gather_acceleration);
                    lin_vel.x = delta.x;
                    lin_vel.z = delta.z;
                }
            }
        }
    }
}

fn slow_xp_drops(time: Res<Time>, mut q_npc: Query<&mut LinearVelocity, With<XpDrop>>) {
    for mut lin_vel in &mut q_npc {
        let speed = lin_vel.length();
        if speed > f32::EPSILON {
            let dir = lin_vel.normalize_or_zero();
            lin_vel.0 = (speed - time.delta_seconds() * 5.).max(0.) * dir;
        }
    }
}

fn cleanup_xp_drops(q_npc: Query<Entity, With<XpDrop>>, mut cmd: Commands) {
    for entity in &q_npc {
        cmd.entity(entity).despawn_recursive();
    }
}
