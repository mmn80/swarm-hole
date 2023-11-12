use bevy::prelude::*;
use bevy_hanabi::prelude::*;

pub struct VfxPlugin;

impl Plugin for VfxPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Vfx>()
            .add_event::<DamageParticlesEvent>()
            .add_systems(Startup, setup_vfx)
            .add_systems(Update, update_vfx);
    }
}

#[derive(Resource, Default)]
pub struct Vfx {
    pub damage_particles: Handle<EffectAsset>,
}

fn setup_vfx(mut vfx: ResMut<Vfx>, mut effects: ResMut<Assets<EffectAsset>>, mut cmd: Commands) {
    let spawner = Spawner::once(100.0.into(), false);

    let writer = ExprWriter::new();

    let age = writer.lit(0.).expr();
    let init_age = SetAttributeModifier::new(Attribute::AGE, age);
    let lifetime = writer.lit(0.5).expr();
    let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, lifetime);

    let drag = writer.lit(2.).expr();
    let update_drag = LinearDragModifier::new(drag);

    let color = writer.prop("spawn_color").expr();
    let init_color = SetAttributeModifier::new(Attribute::COLOR, color);

    let normal = writer.prop("normal");

    let pos = writer.lit(Vec3::ZERO);
    let init_pos = SetAttributeModifier::new(Attribute::POSITION, pos.expr());

    let tangent = writer.lit(Vec3::Z).cross(normal.clone());
    let spread = writer.rand(ScalarType::Float) * writer.lit(2.) - writer.lit(1.);
    let speed = writer.rand(ScalarType::Float) * writer.lit(20.);
    let velocity = (normal + tangent * spread * writer.lit(5.0)).normalized() * speed;
    let init_vel = SetAttributeModifier::new(Attribute::VELOCITY, velocity.expr());

    vfx.damage_particles = effects.add(
        EffectAsset::new(32768, spawner, writer.finish())
            .with_name("damage_particles")
            .with_property("spawn_color", 0xFFFFFFFFu32.into())
            .with_property("normal", Vec3::ZERO.into())
            .init(init_pos)
            .init(init_vel)
            .init(init_age)
            .init(init_lifetime)
            .init(init_color)
            .update(update_drag)
            .render(SetSizeModifier {
                size: Vec2::splat(0.1).into(),
                screen_space_size: false,
            }),
    );

    cmd.spawn(ParticleEffectBundle::new(vfx.damage_particles.clone()).with_spawner(spawner))
        .insert(Name::new("damage_particles"));
}

#[derive(Event)]
pub struct DamageParticlesEvent {
    pub position: Vec3,
    pub normal: Vec3,
}

fn update_vfx(
    mut ev_damage_particles: EventReader<DamageParticlesEvent>,
    mut effect: Query<(
        &mut CompiledParticleEffect,
        &mut EffectSpawner,
        &mut Transform,
    )>,
) {
    let Ok((mut effect, mut spawner, mut tr_effect)) = effect.get_single_mut() else {
        return;
    };
    for ev in ev_damage_particles.read() {
        tr_effect.translation = ev.position;

        let r = rand::random::<u8>();
        let g = rand::random::<u8>();
        let b = rand::random::<u8>();
        let color = 0xFF000000u32 | (b as u32) << 16 | (g as u32) << 8 | (r as u32);
        effect.set_property("spawn_color", color.into());

        let normal = ev.normal.normalize();
        effect.set_property("normal", normal.into());

        spawner.reset();
    }
}
