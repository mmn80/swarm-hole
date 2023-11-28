use bevy::{ecs::system::Command, prelude::*};
use bevy_xpbd_3d::{math::*, prelude::*, SubstepSchedule, SubstepSet};

use crate::{app::AppState, player::Player};

pub struct MainPhysicsPlugin;

impl Plugin for MainPhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_physics)
            .add_systems(Update, update_physics_paused)
            .add_systems(
                SubstepSchedule,
                kinematic_collision
                    .in_set(SubstepSet::SolveUserConstraints)
                    .run_if(in_state(AppState::Run)),
            );
    }
}

fn setup_physics(mut debug_config: ResMut<PhysicsDebugConfig>) {
    debug_config.enabled = false;
}

fn update_physics_paused(mut time: ResMut<Time<Physics>>, app_state: Res<State<AppState>>) {
    let state = *app_state.get();
    if state != AppState::Run {
        if !time.is_paused() {
            time.pause();
        }
    } else if time.is_paused() {
        time.unpause();
    }
}

pub struct TogglePhysicsDebug;

impl Command for TogglePhysicsDebug {
    fn apply(self, world: &mut World) {
        let mut debug_config = world.resource_mut::<PhysicsDebugConfig>();
        debug_config.enabled = !debug_config.enabled;
    }
}

#[derive(PhysicsLayer)]
pub enum Layer {
    Player,
    NPC,
    Building,
    Ground,
}

pub const ALL_LAYERS: [Layer; 4] = [Layer::Player, Layer::NPC, Layer::Building, Layer::Ground];

fn kinematic_collision(
    collisions: Res<Collisions>,
    mut q_bodies: Query<(&RigidBody, &mut Position, &Rotation)>,
    q_player: Query<&Transform, With<Player>>,
) {
    let player_pos = q_player
        .get_single()
        .ok()
        .map_or(Vec3::ZERO, |p| p.translation);
    for contacts in collisions.iter() {
        if !contacts.during_current_substep {
            continue;
        }
        if let Ok([(rb1, mut position1, rotation1), (rb2, mut position2, _)]) =
            q_bodies.get_many_mut([contacts.entity1, contacts.entity2])
        {
            for manifold in contacts.manifolds.iter() {
                for contact in manifold.contacts.iter() {
                    if contact.penetration <= Scalar::EPSILON {
                        continue;
                    }
                    if rb1.is_kinematic() && !rb2.is_kinematic() {
                        position1.0 -= contact.global_normal1(rotation1) * contact.penetration;
                    } else if rb2.is_kinematic() && !rb1.is_kinematic() {
                        position2.0 += contact.global_normal1(rotation1) * contact.penetration;
                    } else if rb1.is_kinematic() && rb2.is_kinematic() {
                        let mut normal = contact.global_normal1(rotation1);
                        normal.y = 0.;
                        if (position1.0 - player_pos).length() < (position2.0 - player_pos).length()
                        {
                            position2.0 += normal * contact.penetration;
                        } else {
                            position1.0 -= normal * contact.penetration;
                        }
                    }
                }
            }
        }
    }
}
