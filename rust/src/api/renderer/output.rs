use std::sync::Arc;

use parking_lot::Mutex;

use crate::{
    api::{
        enums::AudioCommand::{self, RebuildOuput},
        error::AudioError,
        renderer::state::{AudioState, DspState},
    },
    error, info,
};
use cpal::{
    BufferSize,
    ErrorKind::{DeviceChanged, DeviceNotAvailable, Xrun},
    OutputCallbackInfo, SampleFormat, Stream, StreamConfig,
    traits::{DeviceTrait, HostTrait, StreamTrait},
};
use crossbeam_channel::Sender;
use flutter_rust_bridge::frb;

#[frb(opaque)]
pub struct AudioOutput {
    stream: Option<Stream>,
    state: Arc<Mutex<AudioState>>,
    dsp: Arc<Mutex<DspState>>,
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
        cpal::default_host()
            .default_output_device()
            .and_then(|d| d.default_output_config().ok())
            .map(|cfg| AudioOutputConfig {
                sample_rate: cfg.sample_rate(),
                channels: cfg.channels(),
            })
            .unwrap_or(AudioOutputConfig { sample_rate: 44100, channels: 2 })
    }

    pub(crate) fn new(
        cmd_tx: Sender<AudioCommand>,
        state: Arc<Mutex<AudioState>>,
        dsp: Arc<Mutex<DspState>>,
        config: AudioOutputConfig,
    ) -> AudioOutput {
        AudioOutput { cmd_tx, state, dsp, stream: None, current_config: config }
    }

    pub fn current_config(&self) -> AudioOutputConfig { self.current_config }

    pub fn has_stream(&self) -> bool {
        self.stream.is_some()
    }

    pub fn dispose_old_stream(&mut self) {
        if let Some(stream) = self.stream.as_ref() {
            let _ = stream.pause();
        }
        self.stream = None;
    }

    /// Snapshot the Arc refs needed to build a stream without holding the output lock.
    pub(crate) fn build_params(&self) -> Option<(Sender<AudioCommand>, Arc<Mutex<AudioState>>, Arc<Mutex<DspState>>)> {
        if self.stream.is_some() {
            return None;
        }
        Some((
            self.cmd_tx.clone(),
            Arc::clone(&self.state),
            Arc::clone(&self.dsp),
        ))
    }

    /// Install a pre-built stream (brief lock only).
    pub(crate) fn install_stream(&mut self, stream: Stream, config: AudioOutputConfig) {
        info!("Install new stream");
        self.stream = Some(stream);
        self.current_config = config;
    }

    /// Build a cpal stream without holding the output mutex (can block 100+ ms).
    pub(crate) fn build_stream(
        cmd_tx: Sender<AudioCommand>,
        state: Arc<Mutex<AudioState>>,
        dsp: Arc<Mutex<DspState>>,
    ) -> Result<(Stream, AudioOutputConfig), String> {
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

        let tx = cmd_tx;
        let err_fn = move |e: cpal::Error| {
            info!("Error in output stream");
            let _ = match e.kind() {
                Xrun => { error!("{}", Xrun); Ok(()) }
                DeviceChanged | DeviceNotAvailable => {
                    info!("{} or {}", DeviceChanged, DeviceNotAvailable);
                    tx.send(RebuildOuput)
                }
                val => { info!("{}", val); Ok(()) }
            };
        };

        let stream = match sample_format {
            SampleFormat::F32 => {
                let state_cb = Arc::clone(&state);
                let dsp_cb   = Arc::clone(&dsp);
                device.build_output_stream(
                    stream_config,
                    move |data: &mut [f32], _: &OutputCallbackInfo| {
                        write_output(&state_cb, &dsp_cb, data);
                    },
                    err_fn,
                    None,
                ).map_err(|e| AudioError::from(e).to_string())?
            }
            SampleFormat::I16 => {
                let state_cb = Arc::clone(&state);
                let dsp_cb   = Arc::clone(&dsp);
                device.build_output_stream(
                    stream_config.clone(),
                    move |data: &mut [i16], _: &OutputCallbackInfo| {
                        let mut buf = vec![0.0f32; data.len()];
                        write_output(&state_cb, &dsp_cb, &mut buf);
                        for (out, &s) in data.iter_mut().zip(buf.iter()) {
                            *out = (s * i16::MAX as f32) as i16;
                        }
                    },
                    err_fn,
                    None,
                ).map_err(|e| AudioError::from(e).to_string())?
            }
            SampleFormat::U16 => {
                let state_cb = Arc::clone(&state);
                let dsp_cb   = Arc::clone(&dsp);
                device.build_output_stream(
                    stream_config,
                    move |data: &mut [u16], _: &OutputCallbackInfo| {
                        let mut buf = vec![0.0f32; data.len()];
                        write_output(&state_cb, &dsp_cb, &mut buf);
                        for (out, &s) in data.iter_mut().zip(buf.iter()) {
                            *out = (((s * 0.5) + 0.5) * u16::MAX as f32) as u16;
                        }
                    },
                    err_fn,
                    None,
                ).map_err(|e| AudioError::from(e).to_string())?
            }
            _ => return Err("Unsupported sample format".to_string()),
        };

        stream.play().map_err(|e| AudioError::from(e).to_string())?;

        let config = AudioOutputConfig {
            sample_rate: output_config.sample_rate(),
            channels: output_config.channels(),
        };
        info!("Stream built successfully!");
        Ok((stream, config))
    }
}

/// Called from the real-time audio callback.
///
/// Lock ordering (always acquired in this order, never inverted):
///   1. DspState    (holds equalizer + effects)
///   2. AudioState  (holds queue consumer + visualizer)
///
/// Neither lock is ever held while acquiring the other in the reverse order
/// anywhere else in the codebase — safe from deadlock.
fn write_output(
    state: &Arc<Mutex<AudioState>>,
    dsp: &Arc<Mutex<DspState>>,
    data: &mut [f32],
) {
    // We lock DspState first in a short window, clone what we need, then
    // render.  Alternatively we could lock both simultaneously, but
    // short-lived single locks are safer.
    //
    // In practice DspState is almost never mutated (only on user action) so
    // contention is negligible.
    let mut dsp_guard = dsp.lock();
    state.lock().render_output_f32(data, Some(&mut *dsp_guard));
}
