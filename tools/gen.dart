import 'dart:io';

import 'package:ffigen/ffigen.dart';

void main() {
  final packageRoot = Platform.script.resolve('../');
  FfiGenerator(
    output: Output(dartFile: packageRoot.resolve('lib/audiopc.g.dart')),
    // Optional. Where to look for header files.
    headers: Headers(entryPoints: [packageRoot.resolve('rust_backend/bindings.h')]),
    // Optional. What functions to generate bindings for.
    functions: Functions.includeSet({
      'audiopc_default_output_sample_rate',
      'audiopc_default_output_channels',
      'audiopc_output_device_count',
      'audiopc_set_source_path',
      'audiopc_set_source_url',
      'audiopc_set_source_memory',
      'audiopc_play',
      'audiopc_pause',
      'audiopc_stop',
      'audiopc_set_volume',
      'audiopc_set_lowpass_hz',
      'audiopc_set_max_queue_seconds',
      'audiopc_get_max_queue_seconds',
      'audiopc_buffered_samples',
      'audiopc_buffered_millis',
      'audiopc_is_playing',
      'audiopc_is_source_loaded',
      'audiopc_seek_millis',
      'audiopc_position_millis',
      'audiopc_duration_millis',
    }),
  ).generate();
}
