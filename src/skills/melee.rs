use bevy::prelude::*;
use bevy_xpbd_3d::prelude::*;

use crate::{app::AppState, npc::Npc, physics::Layer};

use super::{health::TakeDamageEvent, AddSkillEvent, Skill};

pub struct MeleePlugin;

impl Plugin for MeleePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Melee>().add_systems(
            Update,
            (add_melee, update_melee).run_if(in_state(AppState::Run)),
        );
    }
}

#[derive(Component, Reflect)]
pub struct Melee {
    pub config: MeleeConfig,
}

#[derive(Copy, Clone, Reflect)]
pub struct MeleeConfig {
    pub range: f32,
    pub dps: u32,
}

fn add_melee(mut ev_add_skill: EventReader<AddSkillEvent>, mut cmd: Commands) {
    for ev in ev_add_skill.read() {
        if let AddSkillEvent {
            skill: Skill::Melee(config),
            agent,
        } = ev
        {
            cmd.entity(*agent).insert(Melee { config: *config });
        }
    }
}

fn update_melee(
    time: Res<Time>,
    q_space: SpatialQuery,
    q_melee: Query<(&Melee, &Transform), With<Npc>>,
    mut ev_take_damage: EventWriter<TakeDamageEvent>,
) {
    for (melee, tr_npc) in &q_melee {
        let pos = tr_npc.translation;
        for player_ent in q_space.shape_intersections(
            &Collider::ball(melee.config.range),
            pos,
            Quat::default(),
            SpatialQueryFilter::new().with_masks([Layer::Player]),
        ) {
            ev_take_damage.send(TakeDamageEvent {
                target: player_ent,
                damage: time.delta_seconds() * melee.config.dps as f32,
            });
        }
    }
}
