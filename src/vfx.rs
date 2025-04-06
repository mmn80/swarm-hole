use bevy::prelude::*;
use bevy_hanabi::prelude::*;

use crate::app::AppState;

pub struct VfxPlugin;

impl Plugin for VfxPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Vfx>()
            .add_event::<DamageParticlesEvent>()
            .add_systems(Startup, setup_vfx)
            .add_systems(Update, update_vfx.run_if(in_state(AppState::Run)));
    }
}

#[derive(Resource, Default)]
pub struct Vfx {
    pub damage_particles: Handle<EffectAsset>,
}

fn setup_vfx(mut vfx: ResMut<Vfx>, mut effects: ResMut<Assets<EffectAsset>>, mut cmd: Commands) {
    let spawner = SpawnerSettings::once(40.0.into())
        .with_emit_on_start(false);

    let writer = ExprWriter::new();
    let age = writer.lit(0.).expr();
    let init_age = SetAttributeModifier::new(Attribute::AGE, age);
    let lifetime = writer.lit(0.15).expr();
    let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, lifetime);
    let drag = writer.lit(2.).expr();
    let update_drag = LinearDragModifier::new(drag);
    let spawn_color = writer.add_property("spawn_color", Vec4::splat(1.0).into());
    let color = writer.prop(spawn_color).expr();
    let init_color = SetAttributeModifier::new(Attribute::HDR_COLOR, color);
    let normal = writer.add_property("normal", Vec3::ZERO.into());
    let normal = writer.prop(normal);
    let pos = writer.lit(Vec3::ZERO);
    let init_pos = SetAttributeModifier::new(Attribute::POSITION, pos.expr());
    let tangent = writer.lit(Vec3::Y).cross(normal.clone());
    let spread = writer.rand(ScalarType::Float) * writer.lit(2.) - writer.lit(1.);
    let speed = writer.rand(ScalarType::Float) * writer.lit(40.);
    let velocity = (normal + tangent * spread * writer.lit(5.0)).normalized() * speed;
    let init_vel = SetAttributeModifier::new(Attribute::VELOCITY, velocity.expr());

    vfx.damage_particles = effects.add(
        EffectAsset::new(32768, spawner, writer.finish())
            .with_name("damage_particles")
            .init(init_pos)
            .init(init_vel)
            .init(init_age)
            .init(init_lifetime)
            .init(init_color)
            .update(update_drag)
            .render(OrientModifier {
                mode: OrientMode::ParallelCameraDepthPlane,
                rotation: None,
            })
            .render(SetSizeModifier {
                size: CpuValue::Single((Vec2::splat(0.05), 0.).into()),
            }),
    );

    cmd.spawn((
        ParticleEffect::new(vfx.damage_particles.clone()),
        EffectProperties::default(),
        Name::new("damage_particles"),
    ));
}

#[derive(Event)]
pub struct DamageParticlesEvent {
    pub position: Vec3,
    pub normal: Vec3,
    pub color: Color,
}

fn update_vfx(
    mut ev_damage_particles: EventReader<DamageParticlesEvent>,
    mut effect: Query<(
        &mut EffectProperties,
        &mut EffectSpawner,
        &mut Transform,
    )>,
) {
    let Ok((mut effect, mut effect_spawner, mut tr_effect)) = effect.get_single_mut() else {
        return;
    };
    for ev in ev_damage_particles.read() {
        tr_effect.translation = ev.position;

        effect.set(
            "spawn_color",
            Vec4::from(ev.color.to_linear().to_f32_array()).into(),
        );

        let mut normal = ev.normal;
        normal.y = 0.;
        effect.set("normal", normal.normalize().into());

        effect_spawner.reset();
    }
}
