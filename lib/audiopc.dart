import 'dart:async';

// import 'package:flutter/foundation.dart';

import 'audiopc_platform_interface.dart';

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
typedef PlayerCompletedListener = void Function(bool completed);

class Audiopc {
  Timer? _timer;

  static AudiopcState _state = AudiopcState.none;
  get state => _state;
  static double position = 0.0;
  static double duration = 0.0;
  PlayerStateListener? onStateChanged;

  SamplesListener? onSamplesChanged;

  PositionListener? onPositionChanged;

  DurationListener? onDurationChanged;

  PlayerCompletedListener? onPlayerCompleted;

  Audiopc() {
    _timer = Timer.periodic(const Duration(milliseconds: 10), (timer) {
      getState().then((value) {
        if (onPlayerCompleted != null) {
          if (_state == AudiopcState.playing && value == 5.0) {
            onPlayerCompleted!(true);
          }
        }
        switch (value) {
          case 3.0:
            _state = AudiopcState.playing;
            break;
          case 4.0:
            _state = AudiopcState.paused;
            break;
          case 5.0:
            _state = AudiopcState.stopped;
            break;
          default:
            _state = AudiopcState.none;
        }

        if (value != null && onStateChanged != null) {
          onStateChanged!(_state);
        }
      });

      getCurrentPosition().then((value) {
        if (value != null && onPositionChanged != null) {
          onPositionChanged!(value);
          position = value;
        }
      });

      getDuration().then((value) {
        if (value != null && onDurationChanged != null) {
          onDurationChanged!(value);
          duration = value;
        }
      });
      if (_state == AudiopcState.playing) {
        getSamples().then((value) {
          if (value != null && onSamplesChanged != null) {
            onSamplesChanged!(value);
          }
        });
      }
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

  Future<List<double>?> getSamples() {
    return AudiopcPlatform.instance.getSamples();
  }

  Future<bool?> setRate(double rate) {
    return AudiopcPlatform.instance.setRate(rate);
  }
}
