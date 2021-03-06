use rg3d_sound::{
    buffer::{DataSource, SoundBuffer},
    context::{self, Context},
    effects::{reverb::Reverb, BaseEffect, Effect, EffectInput},
    hrtf::HrirSphere,
    math::{mat4::Mat4, quat::Quat, vec3::Vec3},
    renderer::{hrtf::HrtfRenderer, Renderer},
    source::{generic::GenericSourceBuilder, spatial::SpatialSourceBuilder, SoundSource, Status},
};
use std::{
    thread,
    time::{self, Duration},
};

fn main() {
    let hrir_sphere =
        HrirSphere::from_file("examples/data/IRC_1002_C.bin", context::SAMPLE_RATE).unwrap();

    // Initialize new sound context with default output device.
    let context = Context::new().unwrap();

    // Set HRTF renderer instead of default for binaural sound.
    context
        .lock()
        .unwrap()
        .set_renderer(Renderer::HrtfRenderer(HrtfRenderer::new(hrir_sphere)));

    let base_effect = BaseEffect::default();

    // Create reverb effect and set its decay time.
    let mut reverb = Reverb::new(base_effect);
    reverb.set_decay_time(Duration::from_secs_f32(10.0));
    let reverb_handle = context.lock().unwrap().add_effect(Effect::Reverb(reverb));

    // Create some sounds.
    let sound_buffer =
        SoundBuffer::new_generic(DataSource::from_file("examples/data/door_open.wav").unwrap())
            .unwrap();
    let source = SpatialSourceBuilder::new(
        GenericSourceBuilder::new(sound_buffer)
            .with_status(Status::Playing)
            .build()
            .unwrap(),
    )
    .build_source();
    let door_sound = context.lock().unwrap().add_source(source);

    // Each sound source must be attached to effect, otherwise sound won't be passed to effect
    // and you'll hear sound without any difference.
    context
        .lock()
        .unwrap()
        .effect_mut(reverb_handle)
        .add_input(EffectInput::direct(door_sound));

    let sound_buffer =
        SoundBuffer::new_generic(DataSource::from_file("examples/data/drop.wav").unwrap()).unwrap();
    let source = SpatialSourceBuilder::new(
        GenericSourceBuilder::new(sound_buffer)
            .with_status(Status::Playing)
            .with_looping(true)
            .build()
            .unwrap(),
    )
    .build_source();
    let drop_sound_handle = context.lock().unwrap().add_source(source);

    context
        .lock()
        .unwrap()
        .effect_mut(reverb_handle)
        .add_input(EffectInput::direct(drop_sound_handle));

    // Move sound around listener for some time.
    let start_time = time::Instant::now();
    let mut angle = 0.0f32;
    while (time::Instant::now() - start_time).as_secs() < 360 {
        // Separate scope for update to make sure that mutex lock will be released before
        // thread::sleep will be called so context can actually work in background thread.
        {
            let mut context = context.lock().unwrap();

            if let SoundSource::Spatial(sound) = context.source_mut(drop_sound_handle) {
                let axis = Vec3::new(0.0, 1.0, 0.0);
                let rotation_matrix =
                    Mat4::from_quat(Quat::from_axis_angle(axis, angle.to_radians()));
                sound.set_position(&rotation_matrix.transform_vector(Vec3::new(0.0, 0.0, 1.0)));
            }

            angle += 1.6;

            println!("Sound render time {:?}", context.full_render_duration());
        }

        // Limit rate of context updates.
        thread::sleep(Duration::from_millis(100));
    }
}
