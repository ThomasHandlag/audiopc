import 'dart:async';
import 'dart:ffi' as ffi;

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

class AudiopcPlayer {
  static bool _ok(int code) => code == 0;

  bool setFileSource(String path) {
    final ptr = path.toNativeUtf8().cast<ffi.Char>();
    try {
      return _ok(bindings.audiopc_set_source_path(ptr));
    } finally {
      calloc.free(ptr);
    }
  }

  final StreamController positionStreamController =
      StreamController.broadcast();

  AudiopcPlayer() {
    Timer.periodic(const Duration(milliseconds: 100), (_) {
      positionStreamController.add(positionMillis);
    });
  }

  bool setUrlSource(String url) {
    final ptr = url.toNativeUtf8().cast<ffi.Char>();
    try {
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

  bool play() => _ok(bindings.audiopc_play());
  bool pause() => _ok(bindings.audiopc_pause());
  bool stop() => _ok(bindings.audiopc_stop());

  bool setVolume(double value) => _ok(bindings.audiopc_set_volume(value));

  bool setLowPassHz(double hz) => _ok(bindings.audiopc_set_lowpass_hz(hz));

  int get bufferedSamples => bindings.audiopc_buffered_samples();
  int get positionMillis => bindings.audiopc_position_millis();
  int get durationMillis => bindings.audiopc_duration_millis();

  void dispose() {
    stop();
    positionStreamController.close();
  }
}