use bevy::prelude::*;
use bevy_xpbd_3d::{math::*, prelude::*, SubstepSchedule, SubstepSet};

use crate::debug_ui::{DebugUiCommand, DebugUiEvent};

pub struct MainPhysicsPlugin;

impl Plugin for MainPhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, physics_debug_ui).add_systems(
            SubstepSchedule,
            kinematic_collision.in_set(SubstepSet::SolveUserConstraints),
        );
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
    mut bodies: Query<(&RigidBody, &mut Position, &Rotation)>,
) {
    for contacts in collisions.iter() {
        if !contacts.during_current_substep {
            continue;
        }
        if let Ok([(rb1, mut position1, rotation1), (rb2, mut position2, _)]) =
            bodies.get_many_mut([contacts.entity1, contacts.entity2])
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
                    }
                }
            }
        }
    }
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
