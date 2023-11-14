use bevy::prelude::*;
use bevy_xpbd_3d::{math::*, prelude::*, SubstepSchedule, SubstepSet};

use crate::{
    app::AppState,
    debug_ui::{DebugUiCommand, DebugUiEvent},
    player::Player,
};

pub struct MainPhysicsPlugin;

impl Plugin for MainPhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_physics)
            .add_systems(Update, physics_debug_ui)
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

fn physics_debug_ui(
    mut ev_debug_ui: EventReader<DebugUiEvent>,
    mut debug_config: ResMut<PhysicsDebugConfig>,
) {
    for ev in ev_debug_ui.read() {
        if ev.command == DebugUiCommand::TogglePhysicsDebug {
            debug_config.enabled = !debug_config.enabled;
        }
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
