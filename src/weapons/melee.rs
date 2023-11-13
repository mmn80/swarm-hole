use bevy::prelude::*;
use bevy_xpbd_3d::prelude::*;

use crate::{
    npc::{Health, Npc},
    physics::Layer,
    player::Player,
};

pub struct MeleePlugin;

impl Plugin for MeleePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Melee>()
            .add_systems(Update, update_melee);
    }
}

#[derive(Component, Reflect)]
pub struct Melee {
    pub range: f32,
    pub dps: u32,
}

fn update_melee(
    time: Res<Time>,
    q_space: SpatialQuery,
    q_melee: Query<(&Melee, &Transform), With<Npc>>,
    mut q_player: Query<&mut Health, With<Player>>,
) {
    for (melee, tr_npc) in &q_melee {
        let pos = tr_npc.translation;
        for player_ent in q_space.shape_intersections(
            &Collider::ball(melee.range),
            pos,
            Quat::default(),
            SpatialQueryFilter::new().with_masks([Layer::Player]),
        ) {
            if let Ok(mut health) = q_player.get_mut(player_ent) {
                health.take_damage(time.delta_seconds() * melee.dps as f32);
            }
        }
    }
}
