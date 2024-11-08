import 'dart:async';

// import 'package:flutter/foundation.dart';

import 'audiopc_platform_interface.dart';
import 'dart:developer' show log;

enum AudiopcState {
  closed, // No session.
  ready, // Session was created, ready to open a file.
  openPending, // Session is opening a file.
  started, // Session is playing a file.
  paused, // Session is paused.
  stopped, // Session is stopped (ready to play).
  closing
}

typedef AudioPlayerStateListener = void Function(AudiopcState newState);

class AudiopcWrapper {}

class Audiopc {
  static AudiopcState _state = AudiopcState.closed;
  Timer? _timer;

  Future<double?> getState() {
    return AudiopcPlatform.instance.getState();
  }

  get state => _state;

  AudioPlayerStateListener? onStateChanged;

  Audiopc() {
    _timer = Timer.periodic(const Duration(milliseconds: 10), (timer) {
      getState().then((value) {
        if (value != null && onStateChanged != null) {
          if (_state.index != value) {
            onStateChanged!(_state);
          }
        }

        switch (value) {
          case 1:
            _state = AudiopcState.ready;
            break;

          case 0:
            _state = AudiopcState.closed;
            break;
          case 2:
            _state = AudiopcState.openPending;
          case 3:
            _state = AudiopcState.started;
            break;
          case 4:
            _state = AudiopcState.paused;
            break;
          case 5:
            _state = AudiopcState.stopped;
            break;
          case 6:
            _state = AudiopcState.closing;
            break;
          default:
            _state = AudiopcState.closed;
        }
      });
    });
  }

  void close() {
    _timer?.cancel();
  }

  Future<bool?> setVolume() {
    return AudiopcPlatform.instance.setVolume();
  }

  Future<String?> setSource(String path) {
    String? isSet;
    AudiopcPlatform.instance.setSource(path).then((val) {
      log("Source path is set: ${val.toString()}");
      isSet = val;
    });
    return Future.value(isSet);
  }

  Future<bool?> play() {
    var isPlayed;
    AudiopcPlatform.instance.play().then((value) {
      isPlayed = value;
      log("The audio is played: ${value.toString()}");
    });
    return Future.value(isPlayed);
  }

  Future<bool?> pause() {
    var isPaused;
    AudiopcPlatform.instance.pause().then((value) {
      isPaused = value;
      log("The audio is paused: ${value.toString()}");
    });
    return Future.value(isPaused);
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
