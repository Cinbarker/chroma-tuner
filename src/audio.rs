//! Audio input capture and device management
//!
//! Handles real-time audio capture from input devices, maintains a rolling
//! buffer of samples for pitch analysis, and provides device selection functionality.

use anyhow::Result;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Sample, SampleFormat, Stream, StreamConfig};
use std::sync::{Arc, Mutex};

const SAMPLE_RATE: u32 = 44100;
const BUFFER_SIZE: usize = 8192;

#[derive(Clone)]
pub struct AudioData {
    pub samples: Vec<f32>,
    pub sample_rate: f32,
    pub updated: bool,
}

impl AudioData {
    pub fn new() -> Self {
        Self {
            samples: Vec::with_capacity(BUFFER_SIZE),
            sample_rate: SAMPLE_RATE as f32,
            updated: false,
        }
    }

    pub fn push_samples(&mut self, new_samples: &[f32]) {
        if self.samples.len() + new_samples.len() > BUFFER_SIZE {
            let overflow = (self.samples.len() + new_samples.len()) - BUFFER_SIZE;
            self.samples.drain(0..overflow);
        }

        self.samples.extend_from_slice(new_samples);
        self.updated = true;
    }

    pub fn get_samples(&mut self) -> Vec<f32> {
        self.updated = false;
        self.samples.clone()
    }

    pub fn has_new_data(&self) -> bool {
        self.updated && self.samples.len() >= BUFFER_SIZE / 2
    }
}

pub struct AudioCapture {
    _stream: Stream,
}

impl AudioCapture {
    pub fn new(audio_data: Arc<Mutex<AudioData>>) -> Result<Self> {
        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .ok_or_else(|| anyhow::anyhow!("No input device available"))?;

        Self::new_with_device(audio_data, device)
    }

    pub fn new_with_device(
        audio_data: Arc<Mutex<AudioData>>,
        device: cpal::Device,
    ) -> Result<Self> {
        let config = device.default_input_config()?;
        let actual_sample_rate = config.sample_rate().0 as f32;

        println!("Input device: {}", device.name()?);
        println!("Default input config: {:?}", config);
        println!("Actual sample rate: {} Hz", actual_sample_rate);

        if let Ok(mut audio_data) = audio_data.lock() {
            audio_data.sample_rate = actual_sample_rate;
        }

        let stream = match config.sample_format() {
            SampleFormat::I8 => Self::create_stream::<i8>(&device, &config.into(), audio_data)?,
            SampleFormat::I16 => Self::create_stream::<i16>(&device, &config.into(), audio_data)?,
            SampleFormat::I32 => Self::create_stream::<i32>(&device, &config.into(), audio_data)?,
            SampleFormat::I64 => Self::create_stream::<i64>(&device, &config.into(), audio_data)?,
            SampleFormat::U8 => Self::create_stream::<u8>(&device, &config.into(), audio_data)?,
            SampleFormat::U16 => Self::create_stream::<u16>(&device, &config.into(), audio_data)?,
            SampleFormat::U32 => Self::create_stream::<u32>(&device, &config.into(), audio_data)?,
            SampleFormat::U64 => Self::create_stream::<u64>(&device, &config.into(), audio_data)?,
            SampleFormat::F32 => Self::create_stream::<f32>(&device, &config.into(), audio_data)?,
            SampleFormat::F64 => Self::create_stream::<f64>(&device, &config.into(), audio_data)?,
            _ => return Err(anyhow::anyhow!("Unsupported sample format")),
        };

        stream.play()?;

        Ok(Self { _stream: stream })
    }

    fn create_stream<T>(
        device: &Device,
        config: &StreamConfig,
        audio_data: Arc<Mutex<AudioData>>,
    ) -> Result<Stream>
    where
        T: Sample + cpal::SizedSample + Send + 'static,
        f32: cpal::FromSample<T>,
    {
        let stream = device.build_input_stream(
            config,
            move |data: &[T], _: &cpal::InputCallbackInfo| {
                let samples: Vec<f32> = data
                    .iter()
                    .map(|&sample| f32::from_sample(sample))
                    .collect();

                if let Ok(mut audio_data) = audio_data.lock() {
                    audio_data.push_samples(&samples);
                }
            },
            |err| eprintln!("Audio stream error: {}", err),
            None,
        )?;

        Ok(stream)
    }
}

pub fn get_input_devices() -> Result<Vec<(String, cpal::Device)>> {
    let host = cpal::default_host();
    let mut devices = Vec::new();

    for device in host.input_devices()? {
        if let Ok(name) = device.name() {
            devices.push((name, device));
        }
    }

    Ok(devices)
}

pub fn get_default_input_device_name() -> Result<String> {
    let host = cpal::default_host();
    let device = host
        .default_input_device()
        .ok_or_else(|| anyhow::anyhow!("No default input device"))?;
    Ok(device.name()?)
}
