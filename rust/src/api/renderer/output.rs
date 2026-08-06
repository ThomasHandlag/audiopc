use std::sync::Arc;

use parking_lot::Mutex;

use crate::{
    api::{
        enums::AudioCommand::{self, RebuildOuput},
        error::AudioError,
        renderer::state::AudioState,
    },
    error, info,
};
use cpal::{
    BufferSize, ErrorKind::{DeviceChanged, DeviceNotAvailable, Xrun}, OutputCallbackInfo, SampleFormat, Stream, StreamConfig, traits::{DeviceTrait, HostTrait, StreamTrait},
};
use crossbeam_channel::Sender;
use flutter_rust_bridge::frb;

#[frb(opaque)]
pub struct AudioOutput {
    stream: Option<Stream>,
    state: Arc<Mutex<AudioState>>,
    cmd_tx: Sender<AudioCommand>,
    current_config: AudioOutputConfig,
}

#[derive(Clone, Copy, Debug)]
pub struct AudioOutputConfig {
    pub sample_rate: u32,
    pub channels: u16,
}

impl AudioOutput {
    pub(crate) fn get_output_config() -> AudioOutputConfig {
        let device = cpal::default_host()
            .default_output_device()
            .expect("No ouput device found");

        let output_config = device
            .default_output_config()
            .expect("Could not create output");

        AudioOutputConfig {
            sample_rate: output_config.sample_rate(),
            channels: output_config.channels(),
        }
    }

    pub(crate) fn new(
        cmd_tx: Sender<AudioCommand>,
        state: Arc<Mutex<AudioState>>,
        config: AudioOutputConfig,
    ) -> AudioOutput {
        AudioOutput {
            cmd_tx,
            state,
            stream: None,
            current_config: config,
        }
    }

    pub fn current_config(&self) -> AudioOutputConfig {
        self.current_config
    }

    pub fn dispose_old_stream(&mut self) {
        if let Some(stream) = self.stream.as_ref() {
            let _ = stream.pause();
        }
        self.stream = None;
    }

    pub(crate) fn build(&mut self) -> Result<AudioOutputConfig, String> {
        if self.stream.is_some() {
            let device = cpal::default_host()
                .default_output_device()
                .ok_or_else(|| "No output device found".to_string())?;
            let cfg = device
                .default_output_config()
                .map_err(|e| AudioError::from(e).to_string())?;
            self.current_config = AudioOutputConfig {
                sample_rate: cfg.sample_rate(),
                channels: cfg.channels(),
            };
            return Ok(self.current_config);
        }

        let device = cpal::default_host()
            .default_output_device()
            .ok_or_else(|| "No output device found".to_string())?;

        let output_config = device
            .default_output_config()
            .map_err(|e| AudioError::from(e).to_string())?;

        let sample_format = output_config.sample_format();

        let stream_config = StreamConfig {
            channels: output_config.channels(),
            sample_rate: output_config.sample_rate(),
            buffer_size: BufferSize::Default,
        };

        let tx = self.cmd_tx.clone();
        let state = Arc::clone(&self.state);

        let out_channels = output_config.channels() as usize;

        let err_fn = move |e: cpal::Error| {
            info!("Error in output stream");
            let _ = match e.kind() {
                Xrun => {
                    error!("{}", Xrun);
                    Ok(())
                }
                DeviceChanged | DeviceNotAvailable => {
                    info!("{} or {}", DeviceChanged, DeviceNotAvailable);
                    tx.send(RebuildOuput)
                }
                val => {
                    info!("{}", val);
                    Ok(())
                } // because android AAudio backend does not strigger any device changed so skip this.
            };
        };

        let stream = match sample_format {
            SampleFormat::F32 => device
                .build_output_stream(
                    stream_config,
                    move |data: &mut [f32], _: &OutputCallbackInfo| {
                        write_output_f32(data, out_channels, &state);
                    },
                    err_fn,
                    None,
                )
                .map_err(|e| AudioError::from(e).to_string())?,

            SampleFormat::I16 => device
                .build_output_stream(
                    stream_config.clone(),
                    move |data: &mut [i16], _: &OutputCallbackInfo| {
                        write_output_i16(data, out_channels, &state);
                    },
                    err_fn,
                    None,
                )
                .map_err(|e| AudioError::from(e).to_string())?,

            SampleFormat::U16 => device
                .build_output_stream(
                    stream_config,
                    move |data: &mut [u16], _: &OutputCallbackInfo| {
                        write_output_u16(data, out_channels, &state);
                    },
                    err_fn,
                    None,
                )
                .map_err(|e| AudioError::from(e).to_string())?,

            _ => return Err("Unsupported sample format".to_string()),
        };

        stream.play().map_err(|e| AudioError::from(e).to_string())?;

        self.stream = Some(stream);

        self.current_config = AudioOutputConfig {
            sample_rate: output_config.sample_rate(),
            channels: output_config.channels(),
        };

        info!("Stream built successfully!");

        Ok(self.current_config)
    }
}

fn write_output_f32(data: &mut [f32], channels: usize, state: &Arc<Mutex<AudioState>>) {
    let mut g = state.lock();
    for frame in data.chunks_mut(channels) {
        for out in frame.iter_mut() {
            *out = g.next_sample();
        }
    }
}

fn write_output_i16(data: &mut [i16], channels: usize, state: &Arc<Mutex<AudioState>>) {
    let mut g = state.lock();
    for frame in data.chunks_mut(channels) {
        for out in frame.iter_mut() {
            *out = (g.next_sample() * i16::MAX as f32) as i16;
        }
    }
}

fn write_output_u16(data: &mut [u16], channels: usize, state: &Arc<Mutex<AudioState>>) {
    let mut g = state.lock();
    for frame in data.chunks_mut(channels) {
        for out in frame.iter_mut() {
            *out = (((g.next_sample() * 0.5) + 0.5) * u16::MAX as f32) as u16;
        }
    }
}
