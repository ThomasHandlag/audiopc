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
It provides low-level, high-performance audio playback and processing for Flutter applications.

Designed for developers who want more control over audio pipelines, `audiopc` supports decoding, streaming, and real-time audio processing directly inside the audio callback.

---

## ✨ Features

- 🎵 Play local audio files via native CPAL backend  
- 🌐 Stream audio from direct internet URLs  
- 🧩 Decode multiple formats using **Symphonia**  
- 🎚️ Real-time processing (volume & low-pass filter)  
- 🔊 Access and query active output device configuration  
- ⚡ High-performance Rust core with Flutter FFI bridge

| Feature          | Status |
| ---------------- | ------ |
| Local Playback   | ✅      |
| URL Streaming    | ✅      |
| Format Decoding  | ✅      |
| Audio Processing | ✅      |
| Device Query     | ✅      |

## 📦 Installation

Add to your `pubspec.yaml`:

```yaml
dependencies:
  audiopc: ^0.1.3
```

```kotlin
  // Add these line of code to your MainActivity.kt
  import com.thugbn.audiopc.AudiopcBridge
  AudiopcBridge.init(applicationContext)
```

## Usage

```dart
import 'package:audiopc/audiopc.dart';

final player = AudioPlayer();

player.playSource("youraudio_file");

// Adjust volume
player.setVolume(0.8);

// Apply low-pass filter
player.setRate(0.5);
```

## ⭐ Support

If you find this project useful, consider giving it a ⭐ on GitHub!
