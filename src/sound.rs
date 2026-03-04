use kira::{
    AudioManager, AudioManagerSettings, DefaultBackend, Frame, Panning, StartTime,
    sound::static_sound::{StaticSoundData, StaticSoundSettings},
};
use std::{f32::consts::PI, sync::Arc, time::Duration};

pub fn play_sound() -> Result<AudioManager, Box<dyn std::error::Error>> {
    let mut manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())?;
    // Generate synth samples (0.5 seconds of 440Hz sine)
    let sample_rate = 44100;
    let frequency = 440.0;
    let duration_seconds = 0.5;
    let num_samples = (sample_rate as f32 * duration_seconds) as usize;
    let mut frames = Vec::with_capacity(num_samples);
    for i in 0..num_samples {
        let t = i as f32 / sample_rate as f32;
        let val = (2.0 * PI * frequency * t).sin();
        frames.push(Frame::from_mono(val));
    }
    // Initialize either synth or loaded sound
    // let sound_data = StaticSoundData::from_file("examples/assets/ukelele-dolow.ogg")?;
    let sound_data = StaticSoundData {
        sample_rate,
        frames: Arc::from(frames),
        settings: StaticSoundSettings::default(),
        slice: None,
    };
    // Play out left and right
    manager.play(
        sound_data.clone().with_settings(
            StaticSoundSettings::new()
                .panning(Panning::LEFT)
                .volume(-10.0),
        ),
    )?;
    manager.play(
        sound_data.clone().with_settings(
            StaticSoundSettings::new()
                .panning(Panning::RIGHT)
                .volume(-10.0)
                .start_time(StartTime::Delayed(Duration::from_secs(1))),
        ),
    )?;
    // Keep the manager alive
    Ok(manager)
}
