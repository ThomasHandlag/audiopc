import 'dart:async' show Timer;
import 'dart:convert';
import 'dart:ffi' as ffi;
import 'dart:typed_data';

import 'package:audiopc_interface/audiopc_interface.dart';
import 'package:ffi/ffi.dart';

import '../audiopc.g.dart' as bindings;

/// Native player implementation backed by Rust FFI.
class AudioPlayer with PlayerStateMixin implements AudiopcInterface {
  static bool _ok(int code) => code == 0;

  late final Timer _positionTimer;
  late final Timer _stateTimer;

  /// Reads backend capabilities from the Rust/CPAL layer.
  @override
  AudioBackendInfo getAudioBackendInfo() {
    return AudioBackendInfo(
      defaultOutputSampleRate: bindings.audiopc_default_output_sample_rate(),
      defaultOutputChannels: bindings.audiopc_default_output_channels(),
      outputDeviceCount: bindings.audiopc_output_device_count(),
    );
  }

  /// Creates a player and starts a periodic position publisher.
  AudioPlayer() {
    _positionTimer = Timer.periodic(const Duration(milliseconds: 100), (_) {
      if (state == PlayerState.playing) {
        positionController.add(positionMillis);
      }
    });

    _stateTimer = Timer.periodic(const Duration(seconds: 1), (_) {
      // Polling for state changes is not ideal, but the Rust backend does not currently support callbacks.
      // In a future iteration, we could add a callback mechanism to notify Dart of state changes immediately.
      final stateCode = bindings.audiopc_get_player_state();
      if (stateCode >= 0 && stateCode < PlayerState.values.length) {
        setState(PlayerState.values[stateCode]);
      }
    });
  }

  /// Sets a local file path as the active source.
  @override
  bool setFileSource(String path) {
    final ptr = path.toNativeUtf8().cast<ffi.Char>();
    try {
      setState(PlayerState.idle);
      return _ok(bindings.audiopc_set_source_path(ptr));
    } finally {
      calloc.free(ptr);
    }
  }

  /// Sets a direct URL as the active source.
  @override
  bool setUrlSource(String url) {
    final ptr = url.toNativeUtf8().cast<ffi.Char>();
    try {
      setState(PlayerState.idle);
      return _ok(bindings.audiopc_set_source_url(ptr));
    } finally {
      calloc.free(ptr);
    }
  }

  /// Sets an in-memory byte buffer as the active source.
  @override
  bool setMemorySource(List<int> data) {
    final ptr = malloc.allocate<ffi.Uint8>(data.length);
    try {
      final byteList = ptr.asTypedList(data.length);
      byteList.setAll(0, data);
      return _ok(bindings.audiopc_set_source_memory(ptr, data.length));
    } finally {
      malloc.free(ptr);
    }
  }

  /// Seeks to a playback position in milliseconds.
  @override
  bool seek(int positionMillis) {
    final ok = _ok(bindings.audiopc_seek_millis(positionMillis));
    if (ok) {
      positionController.add(positionMillis);
    }
    return ok;
  }

  /// Starts or resumes playback.
  @override
  bool play() {
    final ok = _ok(bindings.audiopc_play());
    if (ok) {
      setState(PlayerState.playing);
    }
    return ok;
  }

  /// Pauses active playback.
  @override
  bool pause() {
    final ok = _ok(bindings.audiopc_pause());
    if (ok) {
      setState(PlayerState.paused);
    }
    return ok;
  }

  /// Stops playback and resets to the idle state.
  @override
  bool stop() {
    final ok = _ok(bindings.audiopc_stop());
    if (ok) {
      setState(PlayerState.stopped);
    }
    return ok;
  }

  /// Convenience method to play a source directly from a URL or file path.
  void playSource(String source) {
    if (source.startsWith('http://') || source.startsWith('https://')) {
      setUrlSource(source);
    } else {
      setFileSource(source);
    }
    play();
  }

  /// Convenience method to play directly from an in-memory byte buffer.
  bool playMemory(Uint8List data) {
    final ok = setMemorySource(data);
    if (ok) {
      play();
    }
    return ok;
  }

  /// Sets output gain where 1.0 is the nominal level.
  @override
  bool setVolume(double value) => _ok(bindings.audiopc_set_volume(value));

  /// Sets low-pass cutoff in Hz. Use 0 to disable filtering.
  @override
  bool setLowPassHz(double hz) => _ok(bindings.audiopc_set_lowpass_hz(hz, 10));

  /// Number of decoded samples waiting in the native buffer.
  @override
  int get bufferedSamples => bindings.audiopc_buffered_samples();

  /// Current playback position in milliseconds.
  @override
  int get positionMillis => bindings.audiopc_position_millis();

  /// Total media duration in milliseconds, or a negative value if unknown.
  @override
  int get durationMillis => bindings.audiopc_duration_millis();

  /// Number of visualizer samples ready to be copied.
  @override
  int get visualizerAvailableSamples =>
      bindings.audiopc_visualizer_available_samples();

  /// Visualizer sample rate reported by the backend.
  @override
  int get visualizerSampleRate => bindings.audiopc_visualizer_sample_rate();

  /// Visualizer channel count reported by the backend.
  @override
  int get visualizerChannels => bindings.audiopc_visualizer_channels();

  /// Copies normalized time-domain visualizer samples.
  @override
  List<double> getVisualizerSamples(int maxSamples) {
    if (maxSamples <= 0) {
      return const [];
    }

    final ptr = calloc<ffi.Float>(maxSamples);
    try {
      final copied = bindings.audiopc_copy_visualizer_samples(ptr, maxSamples);
      if (copied <= 0) {
        return const [];
      }
      final raw = ptr.asTypedList(copied);
      return List<double>.generate(
        copied,
        (index) => raw[index].toDouble(),
        growable: false,
      );
    } finally {
      calloc.free(ptr);
    }
  }

  /// Copies normalized frequency-domain bars for a spectrum view.
  @override
  List<double> getVisualizerSpectrum(int maxBars) {
    if (maxBars <= 0) {
      return const [];
    }

    final ptr = calloc<ffi.Float>(maxBars);
    try {
      final copied = bindings.audiopc_copy_visualizer_spectrum(ptr, maxBars);
      if (copied <= 0) {
        return const [];
      }

      final raw = ptr.asTypedList(copied);
      return List<double>.generate(
        copied,
        (index) => raw[index].toDouble(),
        growable: false,
      );
    } finally {
      calloc.free(ptr);
    }
  }

  /// Retrieves the current metadata snapshot from the native backend.
  @override
  Future<MetaData> getMetadata(String url) async {
    const maxLen = 8192; // Must match Rust buffer size
    final ptr = calloc<ffi.Char>(maxLen);
    final urlPtr = url.toNativeUtf8();
    try {
      if (ptr == ffi.nullptr) {
        throw Exception('Failed to retrieve metadata');
      }
      final result = bindings.audiopc_get_metadata(ptr, maxLen, urlPtr.cast());
      if (result < 0) {
        throw Exception('Failed to retrieve metadata (error code: $result)');
      }
      final jsonString = ptr.cast<Utf8>().toDartString();
      final jsonData = jsonDecode(jsonString) as Map<String, dynamic>;

      return MetaData.fromJson(jsonData);
    } finally {
      calloc.free(ptr);
      calloc.free(urlPtr);
    }
  }

  /// Retrieves the thumbnail image data from the native backend.
  @override
  Future<Uint8List?> getThumbnail(String url, {int maxSize = 20 * 1024 * 1024}) async {
    final urlPtr = url.toNativeUtf8().cast<ffi.Char>();
    try {
      // First, query size (requires API change to support this)
      // For now, allocate a reasonable max size
      const maxThumbnailSize = 20 * 1024 * 1024; // 20MB max
      final ptr = calloc<ffi.Uint8>(maxThumbnailSize);
      if (ptr == ffi.nullptr) {
        throw Exception('Failed to allocate thumbnail buffer');
      }
      try {
        final length = bindings.audiopc_get_thumbnail(
          ptr,
          maxThumbnailSize,
          urlPtr,
        );
        if (length <= 0) {
          return null;
        }
        // Copy data before freeing
        final result = Uint8List(length);
        result.setAll(0, ptr.asTypedList(length));
        return result;
      } finally {
        calloc.free(ptr);
      }
    } finally {
      calloc.free(urlPtr);
    }
  }

  /// Stops playback and releases timers and stream controllers.
  @override
  void dispose() {
    stop();
    _positionTimer.cancel();
    _stateTimer.cancel();
    positionController.close();
    playerStateController.close();
  }

  @override
  Stream<int> get positionStream => positionController.stream;

  @override
  Stream<PlayerState> get stateStream => playerStateController.stream;

  /// Sets playback rate where 1.0 is normal speed.
  bool setPlaybackRate(double rate) => _ok(bindings.audiopc_set_rate(rate));

  /// Gets the current playback rate.
  double get playbackRate => bindings.audiopc_get_rate();

  /// Sets high-pass cutoff in Hz. Use 0 to disable filtering.
  ///
  /// A high-pass filter allows frequencies above the specified cutoff frequency to pass through while attenuating frequencies below it.
  @override
  bool setHighPassHz(double hz) {
    final code = bindings.audiopc_set_high_pass_filter(
      hz,
      0.707,
    ); // Using a default Q of 0.707 for a Butterworth response
    return _ok(code);
  }

  /// Permits frequencies within a specific range while attenuating those outside of it.
  ///
  /// The `min` parameter specifies the lower cutoff frequency in Hz,
  /// while the `max` parameter specifies the upper cutoff frequency in Hz.
  bool setBandPassFilter(double min, double max) {
    final code = bindings.audiopc_set_band_pass_filter(min, max);
    return _ok(code);
  }

  /// Boosts or cuts frequencies around a center frequency in Hz,
  /// with a specified gain in dB and quality factor Q.
  ///
  /// The `centerHz` parameter specifies the center frequency of the peak filter in Hz,
  /// which determines the frequency around which the boost or cut is applied.
  /// The `gainDb` parameter controls the amount of boost or cut applied to frequencies around the center frequency,
  /// where a positive value results in a boost and a negative value results in a cut.
  ///
  /// The `q` parameter controls the quality factor of the filter,
  /// which affects the bandwidth of the boost or cut around the center frequency.
  /// A higher Q value results in a narrower bandwidth,
  /// while a lower Q value results in a wider bandwidth.
  @override
  bool setPeakFilter(double centerHz, double gainDb, double q) {
    final code = bindings.audiopc_set_peak_filter(centerHz, gainDb, q);
    return _ok(code);
  }

  /// Sets a low shelving filter with a specified cutoff frequency in Hz,
  /// gain in dB, and quality factor Q.
  ///
  /// The `cutoffHz` parameter specifies the cutoff frequency of the low shelf filter in Hz,
  /// which determines the point at which the filter starts to boost or cut frequencies.
  ///
  /// The `gainDb` parameter controls the amount of boost or cut applied to frequencies below the cutoff frequency,
  /// where a positive value results in a boost and a negative value results in a cut.
  ///
  /// The `q` parameter controls the quality factor of the filter,
  /// which affects the slope of the boost or cut around the cutoff frequency.
  @override
  bool setLowShelfFilter(double cutoffHz, double gainDb, double q) {
    final code = bindings.audiopc_set_low_shelf_filter(cutoffHz, gainDb, q);
    return _ok(code);
  }

  /// Sets a high shelving filter with a specified cutoff frequency in Hz,
  /// gain in dB, and quality factor Q.
  ///
  /// The `cutoffHz` parameter specifies the cutoff frequency of the high shelf filter in Hz,
  /// which determines the point at which the filter starts to boost or cut frequencies.
  ///
  /// The `gainDb` parameter controls the amount of boost or cut applied to frequencies above the cutoff frequency,
  /// where a positive value results in a boost and a negative value results in a cut.
  ///
  /// The `q` parameter controls the quality factor of the filter,
  /// which affects the slope of the boost or cut around the cutoff frequency.
  /// A higher Q value results in a steeper slope, while a lower Q value results in a gentler slope.
  @override
  bool setHighShelfFilter(double cutoffHz, double gainDb, double q) {
    final code = bindings.audiopc_set_high_shelf_filter(cutoffHz, gainDb, q);
    return _ok(code);
  }

  /// A notch filter (band-rejection filter) passes most frequencies unaltered,
  /// but attenuates those in a specific range to very low levels.
  ///
  /// The `centerHz` parameter specifies the center frequency of the notch in Hz,
  /// and `q` controls the quality factor (bandwidth) of the notch.
  @override
  bool setNotchFilter(double centerHz, double q, _) {
    final code = bindings.audiopc_set_notch_filter(centerHz, q);

    return _ok(code);
  }

  /// Clears all active filters and returns to a clean signal path.
  bool clearFilter() {
    final code = bindings.audiopc_clear_filters();
    return _ok(code);
  }
  
  @override
  bool setBandPassHz(double min, double max) {
    return setBandPassFilter(min, max);
  }
  
  @override
  bool setCombFilter(double delayMs, double feedback, double damp) {
    return true;
  }
  
  @override
  bool setRate(double value) {
    return setPlaybackRate(value);
  }
  
  @override
  bool setReverb(double roomSize, double damping, double wetLevel, double dryLevel) {
    return true;
  }
}
