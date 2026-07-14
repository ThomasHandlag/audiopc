use std::sync::{Arc, Mutex};

use crate::{
    api::{
        enums::AudioCommand::{self, RebuildOuput},
        error::AudioError,
        renderer::state::AudioState,
    },
    debug,
};
use cpal::{
    BufferSize,
    ErrorKind::DeviceChanged,
    OutputCallbackInfo, SampleFormat, Stream, StreamConfig, SupportedStreamConfig,
    traits::{DeviceTrait, HostTrait, StreamTrait},
};
use crossbeam_channel::Sender;
use flutter_rust_bridge::frb;

#[frb(opaque)]
pub struct AudioOutput {
    pub(crate) stream: Option<Stream>,
    pub state: Arc<Mutex<AudioState>>,
    pub cmd_tx: Sender<AudioCommand>,
    pub out_config: SupportedStreamConfig,
}

impl AudioOutput {
    pub fn get_output_config() -> SupportedStreamConfig {
        let device = cpal::default_host()
            .default_output_device()
            .expect("No ouput device found");

        let output_config = device
            .default_output_config()
            .expect("Could not create output");

        output_config
    }

    pub fn new(
        cmd_tx: Sender<AudioCommand>,
        state: Arc<Mutex<AudioState>>,
        out_config: SupportedStreamConfig,
    ) -> AudioOutput {
        AudioOutput {
            cmd_tx,
            state,
            stream: None,
            out_config,
        }
    }

    pub fn build(&mut self) -> Result<(), String> {
        let device = cpal::default_host()
            .default_output_device()
            .expect("No ouput device found");

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
        let sample_rate = output_config.sample_rate();

        let err_fn = move |e: cpal::Error| {
            let _ = match e.kind() {
                DeviceChanged => tx.send(RebuildOuput),
                _ => Ok(()),
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
                        write_output_u16(data, out_channels, &state );
                    },
                    err_fn,
                    None,
                )
                .map_err(|e| AudioError::from(e).to_string())?,

            _ => return Err("Unsupported sample format".to_string()),
        };

        stream.play().map_err(|e| AudioError::from(e).to_string())?;

        self.stream = Some(stream);

        Ok(())
    }

    pub fn get_config() -> Result<StreamConfig, String> {
        let device = cpal::default_host()
            .default_output_device()
            .expect("No ouput device found");

        let output_config = device
            .default_output_config()
            .map_err(|e| AudioError::from(e).to_string())?;

        Ok(StreamConfig {
            channels: output_config.channels(),
            sample_rate: output_config.sample_rate(),
            buffer_size: BufferSize::Default,
        })
    }
}

fn write_output_f32(
    data: &mut [f32],
    channels: usize,
    state: &Arc<Mutex<AudioState>>,
) {
    let mut g = match state.lock() {
        Ok(g) => g,
        Err(_) => {
            debug!("Lock failed; filling silence");
            data.fill(0.0);
            return;
        }
    };
    for frame in data.chunks_mut(channels) {
        for out in frame.iter_mut() {
            *out = g.next_sample(channels);
        }
    }
}

fn write_output_i16(
    data: &mut [i16],
    channels: usize,
    state: &Arc<Mutex<AudioState>>,
) {
    let mut g = match state.lock() {
        Ok(g) => g,
        Err(_) => {
            data.fill(0);
            return;
        }
    };
    for frame in data.chunks_mut(channels) {
        for out in frame.iter_mut() {
            *out = (g.next_sample(channels) * i16::MAX as f32) as i16;
        }
    }
}

fn write_output_u16(
    data: &mut [u16],
    channels: usize,
    state: &Arc<Mutex<AudioState>>,
) {
    let mut g = match state.lock() {
        Ok(g) => g,
        Err(_) => {
            data.fill(u16::MAX / 2);
            return;
        }
    };
    for frame in data.chunks_mut(channels) {
        for out in frame.iter_mut() {
            *out = (((g.next_sample(channels) * 0.5) + 0.5) * u16::MAX as f32) as u16;
        }
    }
}
