use bevy::prelude::*;
use bevy_xpbd_3d::prelude::*;

use crate::{app::AppState, npc::Npc, physics::Layer};

use super::{apply_skill_specs, health::TakeDamageEvent, IsSkill, Skill};

pub struct MeleePlugin;

impl Plugin for MeleePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Melee>().add_systems(
            Update,
            (apply_skill_specs::<Melee>, update_melee).run_if(in_state(AppState::Run)),
        );
    }
}

#[derive(Component, Reflect, Default)]
pub struct Melee {
    pub range: f32,
    pub dps: u32,
}

impl IsSkill for Melee {
    fn skill() -> Skill {
        Skill::Melee
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
            &Collider::ball(melee.range),
            pos,
            Quat::default(),
            SpatialQueryFilter::new().with_masks([Layer::Player]),
        ) {
            ev_take_damage.send(TakeDamageEvent {
                target: player_ent,
                damage: time.delta_seconds() * melee.dps as f32,
            });
        }
    }
}
