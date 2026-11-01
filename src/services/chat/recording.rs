//! Microphone capture is kept off the UI thread; outgoing recordings are standard PCM WAV.
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};

type RecordingResult = Arc<Mutex<Option<Result<Vec<u8>, String>>>>;

#[derive(Clone)]
pub struct Recorder {
    stop: Arc<AtomicBool>,
    result: RecordingResult,
}

impl Recorder {
    pub fn start() -> Self {
        let stop = Arc::new(AtomicBool::new(false));
        let result = Arc::new(Mutex::new(None));
        let flag = stop.clone();
        let output = result.clone();
        std::thread::spawn(move || {
            let recording =
                (|| -> Result<Vec<u8>, String> {
                    let device = cpal::default_host()
                        .default_input_device()
                        .ok_or("services-no-microphone-available")?;
                    let supported = device.default_input_config().map_err(|e| e.to_string())?;
                    let rate = supported.sample_rate();
                    let channels = supported.channels();
                    let config: cpal::StreamConfig = supported.into();
                    let samples = Arc::new(Mutex::new(Vec::<i16>::new()));
                    let errors = Arc::new(Mutex::new(None));
                    let max = rate as usize * channels as usize * 180;
                    let errors_cb = errors.clone();
                    let build_error = move |err: cpal::Error| {
                        *errors_cb.lock().unwrap() = Some(err.to_string());
                    };
                    let stream =
                        match supported.sample_format() {
                            cpal::SampleFormat::F32 => {
                                let data = samples.clone();
                                device.build_input_stream(
                                    config,
                                    move |input: &[f32], _: &_| {
                                        let mut data = data.lock().unwrap();
                                        let available = max.saturating_sub(data.len());
                                        data.extend(input.iter().take(available).map(|v| {
                                            (v.clamp(-1.0, 1.0) * i16::MAX as f32) as i16
                                        }));
                                    },
                                    build_error,
                                    None,
                                )
                            }
                            cpal::SampleFormat::I16 => {
                                let data = samples.clone();
                                device.build_input_stream(
                                    config,
                                    move |input: &[i16], _: &_| {
                                        let mut data = data.lock().unwrap();
                                        let available = max.saturating_sub(data.len());
                                        data.extend(input.iter().take(available).copied());
                                    },
                                    build_error,
                                    None,
                                )
                            }
                            cpal::SampleFormat::U16 => {
                                let data = samples.clone();
                                device.build_input_stream(
                                    config,
                                    move |input: &[u16], _: &_| {
                                        let mut data = data.lock().unwrap();
                                        let available = max.saturating_sub(data.len());
                                        data.extend(
                                            input
                                                .iter()
                                                .take(available)
                                                .map(|v| (*v as i32 - 32768) as i16),
                                        );
                                    },
                                    build_error,
                                    None,
                                )
                            }
                            _ => return Err("services-unsupported-microphone-format".into()),
                        }
                        .map_err(|e| e.to_string())?;
                    stream.play().map_err(|e| e.to_string())?;
                    let start = std::time::Instant::now();
                    while !flag.load(Ordering::Relaxed) && start.elapsed().as_secs() < 180 {
                        if let Some(error) = errors.lock().unwrap().clone() {
                            return Err(error);
                        }
                        std::thread::sleep(std::time::Duration::from_millis(30));
                    }
                    drop(stream);
                    let data = samples.lock().unwrap();
                    if data.is_empty() {
                        return Err("services-the-microphone-did-not-produce-audio".into());
                    }
                    Ok(wav(&data, rate, channels))
                })();
            *output.lock().unwrap() = Some(recording);
        });
        Self { stop, result }
    }
    pub fn stop(&self) {
        self.stop.store(true, Ordering::Relaxed);
    }
    pub fn take(&self) -> Option<Result<Vec<u8>, String>> {
        self.result.lock().unwrap().take()
    }
}

pub fn wav(samples: &[i16], rate: u32, channels: u16) -> Vec<u8> {
    let length = samples.len() as u32 * 2;
    let mut bytes = Vec::with_capacity(length as usize + 44);
    bytes.extend(b"RIFF");
    bytes.extend((36 + length).to_le_bytes());
    bytes.extend(b"WAVEfmt ");
    bytes.extend(16_u32.to_le_bytes());
    bytes.extend(1_u16.to_le_bytes());
    bytes.extend(channels.to_le_bytes());
    bytes.extend(rate.to_le_bytes());
    bytes.extend((rate * channels as u32 * 2).to_le_bytes());
    bytes.extend((channels * 2).to_le_bytes());
    bytes.extend(16_u16.to_le_bytes());
    bytes.extend(b"data");
    bytes.extend(length.to_le_bytes());
    for sample in samples {
        bytes.extend(sample.to_le_bytes());
    }
    bytes
}

#[cfg(test)]
mod tests {
    use super::*;
    use rodio::Source;
    #[test]
    fn recorded_wav_can_be_decoded_by_the_native_audio_player() {
        let samples = vec![0_i16; 48000];
        let decoder = rodio::Decoder::new(std::io::Cursor::new(wav(&samples, 48000, 1))).unwrap();
        assert_eq!(decoder.sample_rate().get(), 48000);
        assert_eq!(decoder.channels().get(), 1);
        assert_eq!(decoder.total_duration().unwrap().as_secs(), 1);
    }
}
