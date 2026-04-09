import 'dart:async';
import 'dart:ffi' as ffi;

import 'package:audiopc/state.dart';
import 'package:ffi/ffi.dart';

import 'audiopc.g.dart' as bindings;

class AudioBackendInfo {
  final int defaultOutputSampleRate;
  final int defaultOutputChannels;
  final int outputDeviceCount;

  const AudioBackendInfo({
    required this.defaultOutputSampleRate,
    required this.defaultOutputChannels,
    required this.outputDeviceCount,
  });

  bool get isAvailable =>
      defaultOutputSampleRate > 0 &&
      defaultOutputChannels > 0 &&
      outputDeviceCount >= 0;
}

AudioBackendInfo getAudioBackendInfo() {
  return AudioBackendInfo(
    defaultOutputSampleRate: bindings.audiopc_default_output_sample_rate(),
    defaultOutputChannels: bindings.audiopc_default_output_channels(),
    outputDeviceCount: bindings.audiopc_output_device_count(),
  );
}

class AudiopcPlayer with StateMixin {
  static bool _ok(int code) => code == 0;

  final StreamController<int> positionStreamController =
      StreamController<int>.broadcast();

  late final Timer _positionTimer;

  AudiopcPlayer() {
    _positionTimer = Timer.periodic(const Duration(milliseconds: 100), (_) {
      positionStreamController.add(positionMillis);
    });
  }

  bool setFileSource(String path) {
    final ptr = path.toNativeUtf8().cast<ffi.Char>();
    try {
      setState(PlayerState.idle);
      return _ok(bindings.audiopc_set_source_path(ptr));
    } finally {
      calloc.free(ptr);
    }
  }

  bool setUrlSource(String url) {
    final ptr = url.toNativeUtf8().cast<ffi.Char>();
    try {
      setState(PlayerState.idle);
      return _ok(bindings.audiopc_set_source_url(ptr));
    } finally {
      calloc.free(ptr);
    }
  }

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

  void seek(int positionMillis) => bindings.audiopc_seek_millis(positionMillis);

  bool play() {
    final isOk = _ok(bindings.audiopc_play());
    if (isOk) {
      setState(PlayerState.playing);
    }
    return isOk;
  }

  bool pause() {
    final isOk = _ok(bindings.audiopc_pause());
    if (isOk) {
      setState(PlayerState.paused);
    }
    return isOk;
  }

  bool stop() {
    final isOk = _ok(bindings.audiopc_stop());
    if (isOk) {
      setState(PlayerState.idle);
    }
    return isOk;
  }

  bool setVolume(double value) => _ok(bindings.audiopc_set_volume(value));

  bool setLowPassHz(double hz) => _ok(bindings.audiopc_set_lowpass_hz(hz));

  int get bufferedSamples => bindings.audiopc_buffered_samples();
  int get positionMillis => bindings.audiopc_position_millis();
  int get durationMillis => bindings.audiopc_duration_millis();
  int get visualizerAvailableSamples =>
      bindings.audiopc_visualizer_available_samples();
  int get visualizerSampleRate => bindings.audiopc_visualizer_sample_rate();
  int get visualizerChannels => bindings.audiopc_visualizer_channels();

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

  void dispose() {
    stop();
    _positionTimer.cancel();
    positionStreamController.close();
    stateStreamController.close();
  }
}
