import 'dart:async';

// import 'package:flutter/foundation.dart';

import 'audiopc_platform_interface.dart';
import 'dart:developer' show log;

enum AudiopcState {
  none, 
  playing, 
  paused, 
  stopped,
}

typedef PlayerStateListener = void Function(AudiopcState newState);
typedef SamplesListener = void Function(List<double> samples);
typedef PositionListener = void Function(double position);
typedef DurationListener = void Function(double duration);

class AudiopcWrapper {}

class Audiopc {
  Timer? _timer;

  static AudiopcState _state = AudiopcState.none;
  get state => _state;
  PlayerStateListener? onStateChanged;

  SamplesListener? onSamplesChanged;

  PositionListener? onPositionChanged;

  DurationListener? onDurationChanged;

  Audiopc() {
    _timer = Timer.periodic(const Duration(milliseconds: 10), (timer) {
      getState().then((value) {
        if (value != null && onStateChanged != null) {
            onStateChanged!(_state);
        }

        switch (value) {
          case 3:
            _state = AudiopcState.playing;
            break;
          case 4:
            _state = AudiopcState.paused;
            break;
          case 5:
            _state = AudiopcState.stopped;
            break;
          default:
            _state = AudiopcState.none;
        }
      });

      getCurrentPosition().then((value) {
        if (value != null && onPositionChanged != null) {
          onPositionChanged!(value);
        }
      });

      getDuration().then((value) {
        if (value != null && onDurationChanged != null) {
          onDurationChanged!(value);
        }
      });

      getSamples().then((value) {
        if (value != null && onSamplesChanged != null) {
          onSamplesChanged!(value);
        }
      });
    });
  }

  void close() {
    _timer?.cancel();
  }

  Future<double?> getState() {
    return AudiopcPlatform.instance.getState();
  }

  Future<bool?> setVolume() {
    return AudiopcPlatform.instance.setVolume();
  }

  Future<String?> setSource(String path) {
    String? isSet;
    AudiopcPlatform.instance.setSource(path).then((val) {
      isSet = val;
    });
    return Future.value(isSet);
  }

  Future<bool?> play() {
   return AudiopcPlatform.instance.play();
  }

  Future<bool?> pause() {
   return AudiopcPlatform.instance.pause();
  }

  Future<double?> getDuration() {
    return AudiopcPlatform.instance.getDuration();
  }

  Future<double?> getCurrentPosition() {
    return AudiopcPlatform.instance.getPosition();
  }

  Future<double?> seek(double position) {
    return AudiopcPlatform.instance.seek(position);
  }
}
