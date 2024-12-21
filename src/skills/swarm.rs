use std::f32::consts::PI;

use bevy::prelude::*;
use avian3d::prelude::*;
use rand::prelude::*;

use crate::{app::AppState, player::Player};

use super::{apply_skill_specs, IsSkill, Skill};

pub struct SwarmPlugin;

impl Plugin for SwarmPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (apply_skill_specs::<Swarm>, move_swarm).run_if(in_state(AppState::Run)),
        );
    }
}

#[derive(Component, Reflect, Default)]
pub struct Swarm {
    pub speed: f32,
    pub range: f32,
}

impl IsSkill for Swarm {
    fn skill() -> Skill {
        Skill::Swarm
    }
}

const ROAM_SPEED: f32 = 0.5;

fn move_swarm(
    mut q_npc: Query<(&Swarm, &Position, &mut LinearVelocity)>,
    q_player: Query<&Position, With<Player>>,
) {
    let Ok(player_pos) = q_player.get_single() else {
        for (_, _, mut lin_vel) in &mut q_npc {
            lin_vel.x = 0.;
            lin_vel.y = 0.;
            lin_vel.z = 0.;
        }
        return;
    };
    let mut rng = thread_rng();
    for (swarm, npc_pos, mut lin_vel) in &mut q_npc {
        lin_vel.y = 0.;
        let delta = Vec2::new(player_pos.x - npc_pos.x, player_pos.z - npc_pos.z);
        let dir = {
            if delta.length() < swarm.range {
                delta.normalize() * swarm.speed
            } else {
                let vel = Vec2::new(lin_vel.x, lin_vel.z);
                let (max_angle, new_vel) = {
                    if vel.length() > ROAM_SPEED / 2. && vel.length() < 2. * ROAM_SPEED {
                        (PI / 45., vel)
                    } else {
                        (PI, Vec2::ONE * ROAM_SPEED)
                    }
                };
                let alpha = rng.gen_range(-max_angle..max_angle);
                let rot = Vec2::new(alpha.cos(), alpha.sin());
                rot.rotate(new_vel)
            }
        };
        lin_vel.x = dir.x;
        lin_vel.z = dir.y;
    }
}
