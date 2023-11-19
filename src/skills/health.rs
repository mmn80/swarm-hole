use bevy::prelude::*;
use bevy_xpbd_3d::prelude::*;
use serde::Deserialize;

use crate::{
    app::{AppState, RunState},
    npc::Npc,
    physics::Layer,
    player::Player,
};

use super::xp_drops::{XpDrop, XpDrops};

pub struct HealthPlugin;

impl Plugin for HealthPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<TakeDamageEvent>().add_systems(
            Update,
            (take_damage, regen_health, die).run_if(in_state(AppState::Run)),
        );
    }
}

#[derive(Component, Reflect, Clone, Debug, Deserialize)]
pub struct HealthRegen {
    pub hp_per_sec: f32,
}

#[derive(Component)]
pub struct Health {
    pub hp: f32,
    pub max_hp: f32,
}

impl Health {
    pub fn new(max_health: f32) -> Self {
        Self {
            hp: max_health,
            max_hp: max_health,
        }
    }
}

#[derive(Event)]
pub struct TakeDamageEvent {
    pub target: Entity,
    pub damage: f32,
}

fn take_damage(mut ev_take_damage: EventReader<TakeDamageEvent>, mut q_health: Query<&mut Health>) {
    for TakeDamageEvent { target, damage } in ev_take_damage.read() {
        if let Ok(mut health) = q_health.get_mut(*target) {
            health.hp = if *damage >= health.hp {
                0.
            } else {
                health.hp - damage
            };
        }
    }
}

fn regen_health(time: Res<Time>, mut q_regen: Query<(&mut Health, &HealthRegen)>) {
    for (mut health, health_regen) in &mut q_regen {
        health.hp =
            (health.hp + health_regen.hp_per_sec * time.delta_seconds()).min(health.max_hp as f32);
    }
}

fn die(
    mut next_state: ResMut<NextState<AppState>>,
    mut run_state: ResMut<RunState>,
    mut meshes: ResMut<Assets<Mesh>>,
    xp_drops: Res<XpDrops>,
    q_npc: Query<(Entity, &Health, &Transform, Option<&Npc>, Has<Player>)>,
    mut cmd: Commands,
) {
    for (npc_ent, health, tr_npc, npc, is_player) in &q_npc {
        if health.hp <= f32::EPSILON {
            if let Some(npc) = npc {
                run_state.live_npcs -= 1;

                let h = XpDrop::get_height(npc.xp_drop);
                let p = tr_npc.translation;
                let id = cmd
                    .spawn((
                        XpDrop(npc.xp_drop),
                        PbrBundle {
                            transform: Transform::from_translation(Vec3::new(p.x, h + 0.02, p.z)),
                            mesh: meshes.add(
                                Mesh::try_from(shape::Icosphere {
                                    radius: h,
                                    subdivisions: 4,
                                })
                                .unwrap(),
                            ),
                            material: (if XpDrop::is_big(npc.xp_drop) {
                                xp_drops.xp_drop_big.clone()
                            } else {
                                xp_drops.xp_drop_small.clone()
                            }),
                            ..default()
                        },
                        RigidBody::Kinematic,
                        Collider::ball(h),
                        CollisionLayers::new([Layer::Building], [Layer::Building, Layer::Player]),
                    ))
                    .id();
                cmd.entity(id)
                    .insert(Name::new(format!("Xp Drop of {} ({id:?})", npc.xp_drop)));

                if run_state.live_npcs == 0 {
                    next_state.set(AppState::Won);
                }
            } else if is_player {
                next_state.set(AppState::Lost);
            }
            cmd.entity(npc_ent).despawn_recursive();
        }
    }
}
