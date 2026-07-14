use std::collections::HashMap;
use std::time::Duration;

use crate::error::AudioError;

/// Metadata about a track, populated from Symphonia tag data.
#[derive(Debug, Clone, Default)]
pub struct TrackMetadata {
    pub title:  Option<String>,
    pub artist: Option<String>,
    pub album:  Option<String>,
    pub track:  Option<u32>,
    pub year:   Option<u32>,
    pub extra:  HashMap<String, String>,
}

/// Device information for device-change events.
#[derive(Debug, Clone)]
pub struct DeviceInfo {
    pub name: String,
    /// Whether this is the system default device.
    pub is_default: bool,
}

/// The canonical event type that every component emits.
///
/// Consumers receive events through a channel obtained from
/// [`Engine::events()`].  The channel is **multi-producer, single-consumer**:
/// each `Engine` instance owns one sender; callers receive a receiver clone.
///
/// All variants are intentionally `Clone` so events can be fanned out to
/// multiple listeners (e.g., UI thread + logging thread) without heap
/// re-allocation.
#[derive(Debug, Clone)]
pub enum AudioEvent {
    // ── Playback lifecycle ────────────────────────────────────────────────
    /// Playback of a source has started (or resumed from the beginning).
    PlaybackStarted,
    /// Playback has been paused.
    PlaybackPaused { position: Duration },
    /// Playback has been resumed from a paused state.
    PlaybackResumed { position: Duration },
    /// The audio source reached its end naturally.
    PlaybackFinished,
    /// Playback was explicitly stopped by the caller.
    PlaybackStopped,
    /// A new track has become the active source.
    TrackChanged { metadata: TrackMetadata },

    // ── Queue ─────────────────────────────────────────────────────────────
    /// The playback queue has been exhausted.
    QueueExhausted,
    /// The queue length has changed.
    QueueUpdated { len: usize },

    // ── Errors & diagnostics ──────────────────────────────────────────────
    /// A non-fatal error occurred; playback may continue.
    Error(AudioError),
    /// A buffer underrun occurred; silence was inserted automatically.
    Underrun { count: u32 },
    /// The decoder emitted a non-fatal warning.
    DecoderWarning(String),

    // ── Network streaming ─────────────────────────────────────────────────
    /// The stream is buffering; `percent` is 0.0–100.0.
    Buffering { percent: f32 },
    /// Buffering has completed and playback is ready.
    BufferingComplete,
    /// A reconnect attempt is in progress.
    Reconnecting { attempt: u32 },
    /// ICY / Shoutcast stream metadata tags received.
    StreamMetadata(HashMap<String, String>),

    // ── Device hotplug ────────────────────────────────────────────────────
    /// A new audio device was connected to the system.
    DeviceAdded(DeviceInfo),
    /// A previously available device was disconnected.
    DeviceRemoved(DeviceInfo),
    /// The system default device has changed.
    DefaultDeviceChanged(DeviceInfo),

    // ── Position (progress bar) ───────────────────────────────────────────
    /// Periodic position update.  Emitted at a configurable cadence.
    Position {
        current: Duration,
        total:   Option<Duration>,
    },
}

// ── Channel helpers ───────────────────────────────────────────────────────────

/// Sender half of the event channel.
pub type EventSender = std::sync::mpsc::Sender<AudioEvent>;

/// Receiver half of the event channel.
pub type EventReceiver = std::sync::mpsc::Receiver<AudioEvent>;

/// Create a new event channel pair.
pub fn event_channel() -> (EventSender, EventReceiver) {
    std::sync::mpsc::channel()
}
