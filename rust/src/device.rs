/// Device management — wraps cpal host/device enumeration.
///
/// `DeviceManager` provides:
/// * Listing output and input devices.
/// * Resolving a preferred device name with fallback to the system default.
/// * A background watcher that detects device plug/unplug events and notifies
///   callers via the shared event channel (see [`crate::events::AudioEvent`]).
///
/// # Thread safety
///
/// `DeviceManager` itself is not `Send` on all cpal backends.  Keep it on
/// the engine thread and communicate via the event channel.

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread;
use std::time::Duration;

use cpal::traits::{DeviceTrait, HostTrait};

use crate::{debug, enums::DEVICE_POLL_INTERVAL_MS, ffi::revise_stream};
use crate::error::AudioError;
use crate::events::{AudioEvent, DeviceInfo, EventSender};
use crate::warn;

// ── Internal helper ───────────────────────────────────────────────────────────

/// Retrieve a device's human-readable name.
fn device_description(device: &cpal::Device) -> Option<String> {
    if let Ok(description) = device.description() {
        Some(description.name().to_owned())
    } else {
        None
    }
}

// ── DeviceManager ─────────────────────────────────────────────────────────────

pub struct DeviceManager {
    host: cpal::Host,
}

impl DeviceManager {
    /// Create a `DeviceManager` using the platform's default audio host
    /// (WASAPI on Windows, CoreAudio on macOS, ALSA/PipeWire on Linux).
    pub fn new() -> Self {
        Self { host: cpal::default_host() }
    }

    /// Use a specific host (e.g., JACK on Linux).
    pub fn with_host(host_id: cpal::HostId) -> Result<Self, AudioError> {
        cpal::host_from_id(host_id)
            .map(|host| Self { host })
            .map_err(|e| AudioError::StreamConfig(e.to_string()))
    }

    // ── Output devices ────────────────────────────────────────────────────

    /// List all available output devices.
    pub fn output_devices(&self) -> Vec<DeviceInfo> {
        let default_name = self
            .host
            .default_output_device()
            .and_then(|d| device_description(&d))
            .unwrap_or_default();

        match self.host.output_devices() {
            Ok(iter) => iter
                .filter_map(|d| {
                    let name       = device_description(&d)?;
                    let is_default = name == default_name;
                    Some(DeviceInfo { name, is_default })
                })
                .collect(),
            Err(_) => Vec::new(),
        }
    }

    /// Resolve an output device by optional name, falling back to the system
    /// default.
    ///
    /// # Errors
    ///
    /// Returns [`AudioError::NoDevice`] if there is no default output device,
    /// or [`AudioError::DeviceNotFound`] if a name was specified but not found.
    pub fn resolve_output(&self, preferred: Option<&str>) -> Result<cpal::Device, AudioError> {
        if let Some(name) = preferred {
            if let Ok(mut devices) = self.host.output_devices() {
                if let Some(d) = devices.find(|d| {
                    device_description(d).map(|n| n == name).unwrap_or(false)
                }) {
                    return Ok(d);
                }
            }
            warn!("Output device '{name}' not found; falling back to default");
        }

        self.host
            .default_output_device()
            .ok_or(AudioError::NoDevice)
    }

    // ── Input devices ─────────────────────────────────────────────────────

    /// List all available input (capture) devices.
    pub fn input_devices(&self) -> Vec<DeviceInfo> {
        let default_name = self
            .host
            .default_input_device()
            .and_then(|d| device_description(&d))
            .unwrap_or_default();

        match self.host.input_devices() {
            Ok(iter) => iter
                .filter_map(|d| {
                    let name       = device_description(&d)?;
                    let is_default = name == default_name;
                    Some(DeviceInfo { name, is_default })
                })
                .collect(),
            Err(_) => Vec::new(),
        }
    }

    /// Resolve an input device by optional name, falling back to the system
    /// default.
    pub fn resolve_input(&self, preferred: Option<&str>) -> Result<cpal::Device, AudioError> {
        if let Some(name) = preferred {
            if let Ok(mut devices) = self.host.input_devices() {
                if let Some(d) = devices.find(|d| {
                    device_description(d).map(|n| n == name).unwrap_or(false)
                }) {
                    return Ok(d);
                }
            }
            warn!("Input device '{name}' not found; falling back to default");
        }

        self.host
            .default_input_device()
            .ok_or(AudioError::NoDevice)
    }

    // ── Default device config shorthand ───────────────────────────────────

    /// Return `(sample_rate, channels)` of the default output device.
    /// Returns `(0, 0)` if no device is available.
    pub fn default_output_format(&self) -> (u32, u16) {
        let Ok(device) = self.resolve_output(None) else { return (0, 0) };
        match device.default_output_config() {
            Ok(cfg) => (cfg.sample_rate(), cfg.channels()),
            Err(_)  => (0, 0),
        }
    }
}

/// Starts a background thread that polls for device changes and emits
/// [`AudioEvent::DeviceAdded`] / [`AudioEvent::DeviceRemoved`] /
/// [`AudioEvent::DefaultDeviceChanged`] events.
///
/// Returns an `Arc<AtomicBool>` that, when set to `true`, stops the watcher.
/// The poll interval is [`DEVICE_POLL_INTERVAL_MS`].
pub fn start_device_watcher(event_tx: EventSender) -> Arc<AtomicBool> {
    let stop_flag       = Arc::new(AtomicBool::new(false));
    let stop_flag_clone = Arc::clone(&stop_flag);

    thread::spawn(move || {
        let manager = DeviceManager::new();

        let mut last_output_count  = manager.output_devices().len();
        let mut last_default_name  = manager
            .resolve_output(None)
            .ok()
            .and_then(|d| device_description(&d))
            .unwrap_or_default();

        loop {
            thread::sleep(Duration::from_millis(DEVICE_POLL_INTERVAL_MS));

            if stop_flag_clone.load(Ordering::Relaxed) {
                break;
            }

            let current_devices = manager.output_devices();
            let current_count   = current_devices.len();
            let current_default = manager
                .resolve_output(None)
                .ok()
                .and_then(|d| device_description(&d))
                .unwrap_or_default();

            // Emit add / remove events based on count heuristic.
            if current_count != last_output_count {
                if current_count > last_output_count {
                    for info in &current_devices {
                        let _ = event_tx.send(AudioEvent::DeviceAdded(info.clone()));
                    }
                } else {
                    for info in &current_devices {
                        let _ = event_tx.send(AudioEvent::DeviceRemoved(info.clone()));
                    }
                }
                last_output_count = current_count;
            }

            // Default device changed.
            if current_default != last_default_name {
                let info = DeviceInfo { name: current_default.clone(), is_default: true };
                let _ = event_tx.send(AudioEvent::DefaultDeviceChanged(info));
                debug!("Default device changed to '{current_default}'");
                revise_stream();
                last_default_name = current_default;
            }
        }
    });

    stop_flag
}
