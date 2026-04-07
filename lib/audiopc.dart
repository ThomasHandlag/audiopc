import 'dart:ffi' as ffi;

import 'package:ffi/ffi.dart';

import 'src/audiopc.g.dart' as bindings;


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

  String get lastError {
    final len = bindings.audiopc_last_error_message_length();
    if (len <= 0) {
      return '';
    }

    final buffer = calloc<ffi.Char>(len + 1);
    try {
      final copied = bindings.audiopc_last_error_message_copy(buffer, len + 1);
      if (copied <= 0) {
        return '';
      }
      return buffer.cast<Utf8>().toDartString();
    } finally {
      calloc.free(buffer);
    }
  }

  bool setFileSource(String path) {
    final ptr = path.toNativeUtf8().cast<ffi.Char>();
    try {
      return _ok(bindings.audiopc_set_source_path(ptr));
    } finally {
      calloc.free(ptr);
    }
  }

  bool setUrlSource(String url) {
    final ptr = url.toNativeUtf8().cast<ffi.Char>();
    try {
      return _ok(bindings.audiopc_set_source_url(ptr));
    } finally {
      calloc.free(ptr);
    }
  }

  bool play() => _ok(bindings.audiopc_play());
  bool pause() => _ok(bindings.audiopc_pause());
  bool stop() => _ok(bindings.audiopc_stop());

  bool setVolume(double value) => _ok(bindings.audiopc_set_volume(value));

  bool setLowPassHz(double hz) => _ok(bindings.audiopc_set_lowpass_hz(hz));

  int get bufferedSamples => bindings.audiopc_buffered_samples();
}
