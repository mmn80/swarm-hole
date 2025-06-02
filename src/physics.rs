use avian3d::{math::*, prelude::*};
use bevy::prelude::*;

use crate::{app::AppState, player::Player};

pub struct MainPhysicsPlugin;

impl Plugin for MainPhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_physics)
            .add_systems(Update, update_physics_paused)
            .add_systems(
                PhysicsSchedule,
                kinematic_collision
                    .in_set(NarrowPhaseSet::Last)
                    .run_if(in_state(AppState::Run)),
            );
    }
}

fn setup_physics(mut config_store: ResMut<GizmoConfigStore>) {
    config_store.config_mut::<PhysicsGizmos>().0.enabled = false;
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
        let mut config_store = world.resource_mut::<GizmoConfigStore>();
        let config = config_store.config_mut::<PhysicsGizmos>().0;
        config.enabled = !config.enabled;
    }
}

#[derive(Default, PhysicsLayer)]
pub enum Layer {
    #[default]
    Default,
    Player,
    NPC,
    Building,
    Ground,
}

fn kinematic_collision(
    collisions: Collisions,
    collider_rbs: Query<&ColliderOf, Without<Sensor>>,
    mut q_bodies: Query<(&RigidBody, &mut Position)>,
    q_player: Query<&Transform, With<Player>>,
) {
    let player_pos = q_player.single().ok().map_or(Vec3::ZERO, |p| p.translation);
    for contacts in collisions.iter() {
        let Ok([&ColliderOf { body: rb1 }, &ColliderOf { body: rb2 }]) =
            collider_rbs.get_many([contacts.collider1, contacts.collider2])
        else {
            continue;
        };
        let Ok([(rb1, mut position1), (rb2, mut position2)]) = q_bodies.get_many_mut([rb1, rb2])
        else {
            continue;
        };
        for manifold in contacts.manifolds.iter() {
            for contact in manifold.points.iter() {
                if contact.penetration <= Scalar::EPSILON {
                    continue;
                }
                if rb1.is_kinematic() && !rb2.is_kinematic() {
                    position1.0 -= manifold.normal * contact.penetration;
                } else if rb2.is_kinematic() && !rb1.is_kinematic() {
                    position2.0 += manifold.normal * contact.penetration;
                } else if rb1.is_kinematic() && rb2.is_kinematic() {
                    let mut normal = manifold.normal;
                    normal.y = 0.;
                    if (position1.0 - player_pos).length() < (position2.0 - player_pos).length() {
                        position2.0 += normal * contact.penetration;
                    } else {
                        position1.0 -= normal * contact.penetration;
                    }
                }
            }
        }
    }
}
