use bevy::prelude::*;
use bevy_xpbd_3d::prelude::*;

use crate::{npc::Npc, physics::Layer, player::Player};

pub struct WeaponsPlugin;

impl Plugin for WeaponsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Laser>()
            .add_systems(Update, laser_target_npc);
    }
}

#[derive(Component, Default, Reflect)]
pub struct Laser {
    pub range: f32,
    pub dps: f32,
    pub duration: f32,
    pub cooldown: f32,
    pub target: Option<Entity>,
    pub ray: Option<Entity>,
    pub time_ended: f32,
}

impl Laser {
    pub fn new(range: f32, dps: f32, duration: f32, cooldown: f32) -> Self {
        Self {
            range,
            dps,
            duration,
            cooldown,
            target: None,
            ray: None,
            time_ended: 0.,
        }
    }
}

#[derive(Component)]
pub struct LaserRay {
    pub source: Entity,
    pub target: Entity,
    pub time_started: f32,
}

fn laser_target_npc(
    time: Res<Time>,
    q_space: SpatialQuery,
    mut q_laser: Query<(&mut Laser, &Transform), With<Player>>,
    q_npc: Query<&Transform, With<Npc>>,
) {
    for (mut laser, tr_player) in &mut q_laser {
        if laser.ray.is_some() || time.elapsed_seconds() - laser.time_ended < laser.cooldown {
            continue;
        }
        let pos = tr_player.translation;
        if let Some((hit_ent, _)) = q_space
            .shape_intersections(
                &Collider::ball(laser.range),
                pos,
                Quat::default(),
                SpatialQueryFilter::new().with_masks([Layer::NPC]),
            )
            .iter()
            .filter_map(|ent| q_npc.get(*ent).ok().map(|tr| (ent, tr.translation)))
            .min_by(|(_, pos1), (_, pos2)| {
                (*pos2 - pos)
                    .length()
                    .partial_cmp(&(*pos1 - pos).length())
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
        {
            laser.target = Some(*hit_ent);
        }
    }
}
