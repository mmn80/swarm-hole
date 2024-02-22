use bevy::prelude::*;
use bevy_xpbd_3d::prelude::*;

use crate::{
    app::{AppState, RunState},
    npc::Npc,
    physics::Layer,
    player::Player,
};

use super::{
    apply_skill_specs,
    xp::{XpDrop, XpDrops},
    EquippedSkills, IsSkill, Skill,
};

pub struct HealthPlugin;

impl Plugin for HealthPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<TakeDamageEvent>().add_systems(
            Update,
            (
                apply_skill_specs::<MaxHealth>,
                apply_skill_specs::<HealthRegen>,
                init_health,
                take_damage,
                regen_health,
                die,
            )
                .run_if(in_state(AppState::Run)),
        );
    }
}

#[derive(Component, Reflect, Default)]
pub struct MaxHealth {
    pub max_hp: u32,
}

impl IsSkill for MaxHealth {
    fn skill() -> Skill {
        Skill::Health
    }
}

#[derive(Component)]
pub struct Health(pub f32);

fn init_health(
    q_health: Query<(Entity, &MaxHealth, &EquippedSkills), Without<Health>>,
    mut cmd: Commands,
) {
    for (ent, max_health, equipped) in &q_health {
        if equipped.is_equipped(Skill::Health) {
            cmd.entity(ent).insert(Health(max_health.max_hp as f32));
        }
    }
}

#[derive(Component, Reflect, Default)]
pub struct HealthRegen {
    pub hp_per_sec: f32,
}

impl IsSkill for HealthRegen {
    fn skill() -> Skill {
        Skill::HealthRegen
    }
}

fn regen_health(time: Res<Time>, mut q_regen: Query<(&mut Health, &MaxHealth, &HealthRegen)>) {
    for (mut health, max_health, health_regen) in &mut q_regen {
        health.0 = (health.0 + health_regen.hp_per_sec * time.delta_seconds())
            .min(max_health.max_hp as f32);
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
            health.0 = if *damage >= health.0 {
                0.
            } else {
                health.0 - damage
            };
        }
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
        if health.0 < 0.9 {
            if let Some(npc) = npc {
                run_state.live_npcs -= 1;

                let h = XpDrop::get_height(npc.xp_drop);
                let p = tr_npc.translation;
                let id = cmd
                    .spawn((
                        XpDrop(npc.xp_drop),
                        PbrBundle {
                            transform: Transform::from_translation(Vec3::new(p.x, h + 0.02, p.z)),
                            mesh: meshes.add(Sphere::new(h).mesh().ico(4).unwrap()),
                            material: (if XpDrop::is_big(npc.xp_drop) {
                                xp_drops.xp_drop_big.clone()
                            } else {
                                xp_drops.xp_drop_small.clone()
                            }),
                            ..default()
                        },
                        RigidBody::Kinematic,
                        Collider::sphere(h),
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
