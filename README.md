# audiopc

Rust-powered Flutter FFI plugin with a CPAL backend for desktop audio playback and processing.

## Features

- Play local audio files through CPAL.
- Stream audio from direct internet URLs.
- Decode common formats through Symphonia.
- Apply volume and low-pass processing in the audio callback.
- Query the active output device configuration.

## Dart API

```dart
import 'package:audiopc/audiopc.dart';

final value = sum(24, 18);
final asyncValue = await sumAsync(24, 18);

final backend = getAudioBackendInfo();
print(backend.defaultOutputSampleRate);
print(backend.defaultOutputChannels);
print(backend.outputDeviceCount);

final player = AudiopcPlayer();
player.setFileSource('C:/music/song.mp3');
player.setVolume(0.8);
player.setLowPassHz(12_000);
player.play();
```

## Build

This package uses a native build hook in `hook/build.dart` and compiles `rust_backend/Cargo.toml` via `native_toolchain_rust`.

For local development:

1. Install Rust toolchain (`rustup`, `cargo`).
2. Run `flutter pub get` in this package.
3. Run example app/tests normally (`flutter test`, `flutter run`, `flutter build ...`).
4. Generate source `dart run tools/gen.dart`

The native asset pipeline will build the Rust library automatically.
