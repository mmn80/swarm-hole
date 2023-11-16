use bevy::prelude::*;
use bevy_xpbd_3d::prelude::*;

use crate::{app::AppState, physics::Layer, player::Player};

pub struct XpDropsPlugin;

impl Plugin for XpDropsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (gather_xp, slow_xp_drops).run_if(in_state(AppState::Run)),
        )
        .add_systems(OnEnter(AppState::Cleanup), cleanup_xp_drops);
    }
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

fn gather_xp(
    time: Res<Time>,
    q_space: SpatialQuery,
    mut q_player: Query<(&Transform, &mut Player)>,
    mut q_xp_drop: Query<(Entity, &Transform, &mut LinearVelocity, &XpDrop)>,
    mut cmd: Commands,
) {
    for (tr_player, mut player) in &mut q_player {
        for ent in q_space
            .shape_intersections(
                &Collider::ball(player.id.gather_range),
                tr_player.translation,
                Quat::default(),
                SpatialQueryFilter::new().with_masks([Layer::Building]),
            )
            .iter()
        {
            if let Ok((ent, tr_xp, mut lin_vel, xp_drop)) = q_xp_drop.get_mut(*ent) {
                let mut delta = tr_player.translation - tr_xp.translation;
                if delta.length() < XpDrop::get_height(xp_drop.0) + 1. {
                    player.xp += xp_drop.0;
                    cmd.entity(ent).despawn_recursive();
                } else {
                    lin_vel.y = 0.;
                    let old_speed = lin_vel.length();
                    delta.y = 0.;
                    delta = delta.normalize()
                        * (old_speed + time.delta_seconds() * player.id.gather_acceleration);
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
