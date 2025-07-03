# Audiopc

[![GitHub license](https://img.shields.io/github/license/ThomasHandlag/audiopc?style=flat-square)](https://github.com/ThomasHandlag/audiopc/blob/main/LICENSE)
[![GitHub stars](https://img.shields.io/github/stars/ThomasHandlag/audiopc?style=flat-square)](https://github.com/ThomasHandlag/audiopc/stargazers)
[![GitHub issues](https://img.shields.io/github/issues/ThomasHandlag/audiopc?style=flat-square)](https://github.com/ThomasHandlag/audiopc/issues)
![GitHub top language](https://img.shields.io/github/languages/top/ThomasHandlag/audiopc)
![Pub Points](https://img.shields.io/pub/points/audiopc)


## 🎶 Seamless Audio Integration for Flutter

Audiopc is a simple and flexible Flutter plugin designed to provide advanced audio capabilities to your applications. Leveraging native code for optimal performance and access to platform-specific features, this plugin allows you to effortlessly integrate a wide range of audio functionalities, from playback to advanced audio processing.

## ✨ Features

- **Audio playback:**
- **Audio metadata:** Read audio metadata
- **Supported platform:** Support Windows, Android
- **Audio formats:** Support for various audio formats e.g., MP3, WAV, AAC

## 🖥️ Platform Compatibility

| Platform | Supported | Notes                |
|----------|:---------:|----------------------|
| Android  |    ✅     | Full support         |
| Windows  |    ✅     | Full support         |
| iOS      |    🚧     | Planned              |
| macOS    |    🚧     | Planned              |
| Linux    |    🚧     | Planned  

## 🚧Expect to achieve

- **Properly stream audio:**
- **Realtime audio processing:** Like equalizer
- **More supported platform: ** Like IOS, MacOS, and Linux

## 🚀 Getting Started

To use this plugin, add `audio` as a dependency in your `pubspec.yaml` file as github path.

```yaml
# dependencies
dependencies:
  flutter:
    sdk: flutter
 #audiopc: ^0.0.1
  audiopc:
    git:
      url: "https://github.com/ThomasHandlag/audiopc"
# end dependencies
```

```dart
// play audio
final player = Audiopc(id: "0");
player.play(file.path);

player.onPositionChanged.listen((position) {
  // listen to position change
});

player.onStateChanged.listen((state) {
    //listen to state changed
});

player.state;

// get audio metadata
AudioMetaData? metadata;

metadata = await player.getMetaData(file.path);
print(metadata?.artist ?? "");

// samples data
player.onSamples.listen((samples) {
  List<double> data = samples;
  // do something with samples
});
```
