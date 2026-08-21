# 🎧 audiopc

[![pub package](https://img.shields.io/pub/v/audiopc.svg)](https://pub.dev/packages/audiopc)
[![likes](https://img.shields.io/pub/likes/audiopc)](https://pub.dev/packages/audiopc/score)
[![license](https://img.shields.io/github/license/thomashandlag/audiopc)](https://github.com/thomashandlag/audiopc/blob/main/LICENSE)
[![stars](https://img.shields.io/github/stars/thomashandlag/audiopc)](https://github.com/thomashandlag/audiopc)
[![issues](https://img.shields.io/github/issues/thomashandlag/audiopc)](https://github.com/thomashandlag/audiopc/issues)
![Pub Points](https://img.shields.io/pub/points/audiopc)

---

## 🚀 Overview

**audiopc** is a Rust-powered Flutter audio plugin built using **FFI** and powered by the **CPAL** backend.  
Provide seamless playback, and advance features like equalizer, audio sample, visualizer.

---

## 💪 Supported platforms

- Windows ✅
- Android ✅
- Linux   ✅
- IOS     🚧
- MacOS   🚧
- Web     🚨

## ✨ Features

- 🎵 Play local audio files via native CPAL backend  
- 🌐 Stream audio from direct internet URLs  
- 🧩 Decode multiple formats using **Symphonia**  
- 🎚️ Real-time processing (volume & effects, fliter, visualizer)  
- 🔊 Access and query active output device configuration  
- ⚡ High-performance with Rust.

| Feature          | Status |
| ---------------- | ------ |
| Local Playback   | ✅     |
| URL Streaming    | ✅     |
| Audio Processing | ✅     |
| Device Query     | ✅     |

## 📦 Installation

### 🧳 Dependency

Add to your `pubspec.yaml`:

```yaml
dependencies:
  audiopc: ^0.1.3
```

### Dart

```dart
void main() async {
  /// Make sure you add this line before you use any api.
  await RustLib.init();

  runApp(YourApp());
}

```

## Usage

Create player instance and play audio source.

```dart
import 'package:audiopc/audiopc.dart';

final player = PcPlayer.instance();

// Set and play audio source.
player.playSource(AudioSource.path(source));

// Adjust volume.
player.setVolume(0.8);

player.setRate(0.5);
```

Get visualizer bars.

```dart
  player.setProcessor(54, 512);

  final bars = player.getBars();
```

## Compability with other packages

**Audiopc** can be used with any packages that implement features like background audio playing, etc...

## ⭐ Support

If you find this project useful, consider giving it a ⭐ on GitHub!

## Contribute

**Feel free to create pull request, issues**;
